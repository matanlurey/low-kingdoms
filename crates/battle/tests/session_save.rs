//! Session command ownership, save integrity, pending decisions, and human-event privacy.
#![cfg(test)]

use low_kingdoms_battle::ai;
use low_kingdoms_battle::content::Content;
use low_kingdoms_battle::session::{HISTORY_LIMIT, Session, SessionError};
use low_kingdoms_rules::command::Command;
use low_kingdoms_rules::event::Event;
use low_kingdoms_rules::model::{CardId, Phase, PlayerId};

fn fixture(cost: u32, power: u32, health: u32, stronghold: u32) -> Content {
    let cards =
        format!("[(id: (1), name: \"Unit\", cost: {cost}, power: {power}, health: {health})]");
    let deck = format!(
        "(cards: [{}], stronghold_health: {stronghold})",
        ["(1)"; 20].join(",")
    );
    Content::parse(&cards, &format!("[{deck},{deck}]")).expect("valid fixture")
}

fn save(session: &Session) -> String {
    session.save().expect("serializable session")
}

fn restored(session: &Session, content: &Content) -> Session {
    let result = Session::restore(content, &save(session)).expect("valid save");
    assert_eq!(save(&result), save(session));
    assert_eq!(result.view(), session.view());
    assert_eq!(result.legal_commands(), session.legal_commands());
    assert_eq!(result.needs_ai(), session.needs_ai());
    result
}

fn advance(session: &mut Session, content: &Content) -> Vec<Event> {
    if session.needs_ai() {
        session
            .step_ai(content)
            .expect("legal deterministic AI command")
    } else {
        let command = ai::choose(&session.view(), &session.legal_commands(), content.cards())
            .expect("human fixture command");
        session.apply_human(&command).expect("legal human command")
    }
}

fn assert_private_events_are_filtered(events: &[Event]) {
    for event in events {
        match event {
            Event::CardsDrawn { player, cards, .. }
            | Event::CardsResourced { player, cards, .. }
                if *player == PlayerId::One =>
            {
                assert!(cards.is_none(), "opposing private card identities leaked");
            }
            Event::ResourcesExhausted { player, cards, .. } if *player == PlayerId::One => {
                assert!(cards.is_none(), "opposing resource identities leaked");
            }
            _ => {}
        }
    }
}

#[test]
fn initialization_exposes_human_view_and_filtered_history_only() {
    let content = Content::load().expect("embedded content");
    let session = Session::new(&content, 0).expect("session");
    let view = session.view();
    assert_eq!(view.player, PlayerId::Zero);
    assert_eq!(view.hand.len(), 6);
    assert_eq!(view.players[1].hand_size, 6);
    assert_eq!(view.players[1].deck_size, 14);
    assert_eq!(session.revision(), 0);
    assert_eq!(session.seed(), 0);
    assert!(session.needs_ai());
    assert_private_events_are_filtered(session.history());
    let encoded_view = serde_json::to_value(&view).expect("view JSON");
    assert!(encoded_view["players"][1].get("hand").is_none());
    assert!(encoded_view["players"][1].get("deck").is_none());
    assert!(
        encoded_view["players"][1]["resources"]
            .get("cards")
            .is_none()
    );
    restored(&session, &content);
}

#[test]
fn invalid_commands_leave_snapshot_log_history_revision_and_decisions_unchanged() {
    let content = Content::load().expect("content");
    let mut session = Session::new(&content, 42).expect("session");
    for command in [
        Command::Pass,
        Command::PlayUnit { card: CardId(999) },
        Command::ChooseInitialResources { cards: vec![] },
    ] {
        let before = save(&session);
        assert!(matches!(
            session.apply_human(&command),
            Err(SessionError::Rules(_))
        ));
        assert_eq!(save(&session), before);
    }
    let events = session.apply_human(&Command::KeepHand).expect("keep");
    assert!(events.is_empty());
    assert_eq!(session.revision(), 1);
    assert!(session.view().decision_submitted);
    let before = save(&session);
    assert!(matches!(
        session.apply_human(&Command::MulliganHand),
        Err(SessionError::Rules(_))
    ));
    assert_eq!(save(&session), before);
}

#[test]
fn both_pending_setup_orders_restore_without_repeating_opponent_decisions() {
    let content = Content::load().expect("content");
    for seed in [0, 1, 42, u64::MAX] {
        let mut first_human = Session::new(&content, seed).expect("session");
        first_human
            .apply_human(&Command::MulliganHand)
            .expect("mulligan");
        let mut resumed = restored(&first_human, &content);
        assert!(resumed.needs_ai());
        assert_eq!(
            first_human.step_ai(&content).expect("AI"),
            resumed.step_ai(&content).expect("AI")
        );
        assert_eq!(save(&first_human), save(&resumed));
        assert_eq!(resumed.view().phase, Phase::InitialResourcing);

        let mut first_ai = Session::new(&content, seed).expect("session");
        assert!(
            first_ai
                .step_ai(&content)
                .expect("AI keep/mulligan")
                .is_empty()
        );
        assert!(!first_ai.needs_ai());
        let mut resumed = restored(&first_ai, &content);
        let before = save(&resumed);
        assert!(matches!(
            resumed.step_ai(&content),
            Err(SessionError::NoAiDecision)
        ));
        assert_eq!(before, save(&resumed));
        assert_eq!(
            first_ai.apply_human(&Command::KeepHand).expect("keep"),
            resumed.apply_human(&Command::KeepHand).expect("keep")
        );
        assert_eq!(save(&first_ai), save(&resumed));
        assert_eq!(resumed.revision(), 2);

        let resource = Command::ChooseInitialResources {
            cards: resumed
                .view()
                .hand
                .iter()
                .take(2)
                .map(|card| card.id)
                .collect(),
        };
        first_ai.apply_human(&resource).expect("resources");
        resumed.apply_human(&resource).expect("resources");
        let mut resumed_again = restored(&resumed, &content);
        assert_eq!(
            first_ai.step_ai(&content).expect("AI resources"),
            resumed_again.step_ai(&content).expect("AI resources")
        );
        assert_eq!(save(&first_ai), save(&resumed_again));
    }
}

#[test]
fn pending_opponent_action_and_regroup_decisions_resume_exactly_once() {
    let content = fixture(1, 2, 3, 30);
    let mut original = Session::new(&content, 7).expect("session");
    original.apply_human(&Command::KeepHand).expect("keep");
    original.step_ai(&content).expect("AI keep");
    original
        .apply_human(&Command::ChooseInitialResources {
            cards: original
                .view()
                .hand
                .iter()
                .take(2)
                .map(|card| card.id)
                .collect(),
        })
        .expect("human resources");
    original.step_ai(&content).expect("AI resources");
    if original.needs_ai() {
        original.step_ai(&content).expect("AI first action");
    }
    let deployment = original
        .legal_commands()
        .into_iter()
        .find(|command| matches!(command, Command::PlayUnit { .. }))
        .expect("affordable human unit");
    original.apply_human(&deployment).expect("deploy");
    assert!(original.needs_ai());
    let mut resumed = restored(&original, &content);
    let revision = resumed.revision();
    assert_eq!(
        original.step_ai(&content).expect("AI action"),
        resumed.step_ai(&content).expect("AI action")
    );
    assert_eq!(resumed.revision(), revision + 1);
    assert_eq!(save(&original), save(&resumed));

    for _ in 0..16 {
        if original.view().phase == Phase::Regroup {
            break;
        }
        if original.needs_ai() {
            original.step_ai(&content).expect("AI action");
        } else {
            original.apply_human(&Command::Pass).expect("human pass");
        }
    }
    assert_eq!(original.view().phase, Phase::Regroup);
    original
        .apply_human(&Command::ChooseRegroupResource { card: None })
        .expect("decline resource");
    assert!(original.view().decision_submitted);
    assert!(original.needs_ai());
    let mut resumed = restored(&original, &content);
    let revision = resumed.revision();
    assert_eq!(
        original.step_ai(&content).expect("AI regroup"),
        resumed.step_ai(&content).expect("AI regroup")
    );
    assert_eq!(resumed.revision(), revision + 1);
    assert_eq!(save(&original), save(&resumed));
    assert_eq!(resumed.view().phase, Phase::Action);
    assert_eq!(resumed.view().round, 2);
}

#[test]
fn restored_complete_matches_have_identical_commands_events_states_and_outcomes() {
    let content = Content::load().expect("content");
    for seed in 0..16 {
        let mut original = Session::new(&content, seed).expect("session");
        let mut resumed = restored(&original, &content);
        for step in 0..512 {
            if original.is_finished() {
                break;
            }
            let left = advance(&mut original, &content);
            let right = advance(&mut resumed, &content);
            assert_eq!(left, right);
            assert_eq!(original.view(), resumed.view());
            assert_eq!(save(&original), save(&resumed));
            assert_private_events_are_filtered(&left);
            assert_private_events_are_filtered(original.history());
            if step % 7 == 0 && !original.is_finished() {
                resumed = restored(&resumed, &content);
            }
        }
        assert!(original.is_finished(), "seed {seed} did not terminate");
        assert_eq!(original.view().outcome, resumed.view().outcome);
        assert!(original.legal_commands().is_empty());
        assert!(!original.needs_ai());
        let finished = restored(&original, &content);
        assert!(finished.is_finished());
        assert!(finished.legal_commands().is_empty());
        assert!(!finished.needs_ai());
    }
}

#[test]
fn save_rejects_incompatible_versions_roles_content_and_ai_profiles() {
    let content = fixture(1, 2, 2, 30);
    let session = Session::new(&content, 42).expect("session");
    let original: serde_json::Value = serde_json::from_str(&save(&session)).expect("save JSON");
    for (field, value, expected) in [
        ("version", serde_json::json!(2), "version"),
        ("human", serde_json::json!("One"), "roles"),
        ("opponent", serde_json::json!("Zero"), "roles"),
        ("content_signature", serde_json::json!("changed"), "content"),
        ("ai_profile_version", serde_json::json!(2), "profile"),
    ] {
        let mut changed = original.clone();
        changed[field] = value;
        let error = Session::restore(&content, &changed.to_string())
            .err()
            .expect("rejected");
        assert!(matches!(
            (&error, expected),
            (SessionError::SaveVersion, "version")
                | (SessionError::SaveRoles, "roles")
                | (SessionError::SaveContent, "content")
                | (SessionError::SaveAiProfile, "profile")
        ));
    }
    for different in [
        fixture(2, 2, 2, 30),
        fixture(1, 3, 2, 30),
        fixture(1, 2, 2, 31),
    ] {
        assert!(matches!(
            Session::restore(&different, &save(&session)),
            Err(SessionError::SaveContent)
        ));
    }
    let changed = fixture(1, 2, 3, 30);
    let mut session = session;
    let before = save(&session);
    assert!(matches!(
        session.step_ai(&changed),
        Err(SessionError::SaveContent)
    ));
    assert_eq!(save(&session), before);
}

#[test]
fn corrupt_snapshot_log_revision_phase_and_private_history_are_rejected() {
    let content = Content::load().expect("content");
    let mut session = Session::new(&content, 42).expect("session");
    session.step_ai(&content).expect("AI decision");
    let original: serde_json::Value = serde_json::from_str(&save(&session)).expect("JSON");
    for mutate in [
        |value: &mut serde_json::Value| value["revision"] = serde_json::json!(999),
        |value: &mut serde_json::Value| value["phase"] = serde_json::json!("Action"),
        |value: &mut serde_json::Value| value["seed"] = serde_json::json!(43),
        |value: &mut serde_json::Value| value["commands"][0]["player"] = serde_json::json!("Zero"),
        |value: &mut serde_json::Value| {
            value["commands"][0]["command"] = if value["commands"][0]["command"] == "KeepHand" {
                serde_json::json!("MulliganHand")
            } else {
                serde_json::json!("KeepHand")
            };
        },
        |value: &mut serde_json::Value| {
            value["history"][0] = serde_json::json!({"CardsDrawn":{
                "player":"One","count":1,"cards":[{"id":21,"definition":1}]
            }});
        },
        |value: &mut serde_json::Value| {
            let mut snapshot: serde_json::Value =
                serde_json::from_str(value["snapshot"].as_str().expect("snapshot text"))
                    .expect("snapshot JSON");
            snapshot["state"]["rng"]["state"] = serde_json::json!(0);
            value["snapshot"] = serde_json::json!(snapshot.to_string());
        },
    ] {
        let mut changed = original.clone();
        mutate(&mut changed);
        assert!(Session::restore(&content, &changed.to_string()).is_err());
    }
    for malformed in ["", "null", "{}", "not JSON", "{\"version\":1}"] {
        assert!(matches!(
            Session::restore(&content, malformed),
            Err(SessionError::SaveFormat)
        ));
    }
    let mut unknown = original;
    unknown["future_field"] = serde_json::json!(true);
    assert!(matches!(
        Session::restore(&content, &unknown.to_string()),
        Err(SessionError::SaveFormat)
    ));
    assert!(matches!(
        Session::restore(&content, &" ".repeat(2 * 1024 * 1024 + 1)),
        Err(SessionError::SaveFormat)
    ));
}

#[test]
fn history_is_bounded_while_full_command_log_still_validates_exact_continuation() {
    let content = fixture(0, 1, 20, 1000);
    let mut session = Session::new(&content, 5).expect("session");
    let mut event_count = session.history().len();
    let mut bounded_restore_exercised = false;
    for _ in 0..2048 {
        if session.is_finished() {
            break;
        }
        event_count += advance(&mut session, &content).len();
        assert!(session.history().len() <= HISTORY_LIMIT);
        if event_count > HISTORY_LIMIT && !bounded_restore_exercised && !session.is_finished() {
            assert_eq!(session.history().len(), HISTORY_LIMIT);
            session = restored(&session, &content);
            bounded_restore_exercised = true;
        }
    }
    assert!(session.is_finished());
    assert!(bounded_restore_exercised);
    assert_private_events_are_filtered(session.history());
}

#[test]
fn independent_fresh_sessions_are_seeded_without_reusing_old_commands_or_history() {
    let content = Content::load().expect("content");
    let mut first = Session::new(&content, 1).expect("session");
    first.apply_human(&Command::KeepHand).expect("keep");
    first.step_ai(&content).expect("AI");
    let second = Session::new(&content, 2).expect("new match");
    assert_eq!(second.revision(), 0);
    assert_eq!(second.view().phase, Phase::Mulligan);
    assert!(!second.view().decision_submitted);
    assert!(second.needs_ai());
    assert_ne!(first.seed(), second.seed());
    assert_ne!(save(&first), save(&second));
}
