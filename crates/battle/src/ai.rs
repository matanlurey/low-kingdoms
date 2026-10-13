//! Stateless deterministic heuristics over an AI player's permitted view and legal commands.

use std::fmt;

use low_kingdoms_rules::command::{AttackTarget, Command};
use low_kingdoms_rules::model::{CardId, DefinitionId, Phase, PlayerId};
use low_kingdoms_rules::view::{PlayerView, UnitView};

use crate::content::CardDefinition;

/// Chooses one supplied legal command without inspecting authoritative state or either deck.
///
/// Setup keeps a hand with an affordable unit and at least four units costing at most four.
/// Initial resources preserve an affordable opening play, then prefer expensive/redundant cards.
/// Regroup adds a resource when it unlocks a play or supports multiple held plays, while preserving
/// the last affordable card. Actions prioritize lethal stronghold attacks, surviving kills,
/// favorable mutual destruction, affordable deployments, and direct stronghold damage. Initiative
/// is claimed only after those useful actions run out; passing remains the final fallback.
///
/// Within each priority, printed statistics determine value, followed by ascending instance IDs.
/// Command enumeration order, clocks, rendering, and OS randomness cannot affect the result.
/// All arithmetic uses widened `u128` scores and saturating remaining Health calculations.
///
/// # Errors
/// Returns an error when no supplied command fits the public phase, when a hand card's definition
/// is absent from the supplied public catalog, or when a supplied command references an own card
/// or public unit absent from the view. The caller must still validate the result through `Game`.
pub fn choose(
    view: &PlayerView,
    legal: &[Command],
    catalog: &[CardDefinition],
) -> Result<Command, AiError> {
    let hand = view
        .hand
        .iter()
        .map(|card| definition(catalog, card.definition).map(|definition| (card.id, definition)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut best: Option<(Evaluation, &Command)> = None;
    for command in legal {
        if let Some(evaluation) = evaluate(view, command, &hand)?
            && best.as_ref().is_none_or(|(current, _)| {
                evaluation.score > current.score
                    || (evaluation.score == current.score && evaluation.key < current.key)
            })
        {
            best = Some((evaluation, command));
        }
    }
    best.map(|(_, command)| command.clone())
        .ok_or(AiError::NoLegalCommand)
}

struct Evaluation {
    score: (u8, u128, u128),
    key: CommandKey,
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum CommandKey {
    Keep,
    Mulligan,
    Initial(Vec<CardId>),
    Regroup(Option<CardId>),
    Play(CardId),
    AttackUnit(CardId, CardId),
    AttackStronghold(CardId, PlayerId),
    Claim,
    Pass,
}

fn evaluate(
    view: &PlayerView,
    command: &Command,
    hand: &[(CardId, &CardDefinition)],
) -> Result<Option<Evaluation>, AiError> {
    let score = match (view.phase, command) {
        (Phase::Mulligan, Command::KeepHand) => (u8::from(keep_hand(hand)) + 1, 0, 0),
        (Phase::Mulligan, Command::MulliganHand) => (u8::from(!keep_hand(hand)) + 1, 0, 0),
        (Phase::InitialResourcing, Command::ChooseInitialResources { cards }) => {
            let preserves_play = hand
                .iter()
                .any(|(id, card)| !cards.contains(id) && card.cost <= 2);
            let value = cards.iter().try_fold(0_u128, |total, id| {
                held_definition(hand, *id).map(|card| total + resource_value(hand, card, 2))
            })?;
            (
                1,
                cards.len() as u128,
                (u128::from(preserves_play) << 96) + value,
            )
        }
        (Phase::Regroup, Command::ChooseRegroupResource { card }) => {
            let resources = own_player(view).resources.total as u128;
            let needs_resource = hand
                .iter()
                .any(|(_, card)| u128::from(card.cost) > resources)
                || (hand.len() > 1
                    && hand
                        .iter()
                        .map(|(_, card)| u128::from(card.cost))
                        .sum::<u128>()
                        > resources);
            match card {
                Some(id) => {
                    let definition = held_definition(hand, *id)?;
                    (
                        u8::from(needs_resource) + 1,
                        resource_value(hand, definition, resources + 1),
                        0,
                    )
                }
                None => (u8::from(!needs_resource) + 1, 0, 0),
            }
        }
        (Phase::Action, Command::PlayUnit { card }) => {
            let definition = held_definition(hand, *card)?;
            (
                4,
                (u128::from(definition.cost) << 64)
                    + (u128::from(definition.power) << 32)
                    + u128::from(definition.health),
                0,
            )
        }
        (Phase::Action, Command::Attack { attacker, target }) => {
            let attacker = own_player(view)
                .units
                .iter()
                .find(|unit| unit.card.id == *attacker)
                .ok_or(AiError::MissingVisibleCard(*attacker))?;
            match target {
                AttackTarget::Stronghold(player) => {
                    let defender = public_player(view, *player);
                    let remaining = u64::from(defender.stronghold_health)
                        .saturating_sub(defender.stronghold_damage);
                    if u64::from(attacker.power) >= remaining {
                        (7, u128::from(u32::MAX - attacker.power), 0)
                    } else if attacker.power > 0 {
                        (3, u128::from(attacker.power), u128::from(attacker.damage))
                    } else {
                        return Ok(None);
                    }
                }
                AttackTarget::Unit(id) => {
                    let defender = public_player(view, view.player.opponent())
                        .units
                        .iter()
                        .find(|unit| unit.card.id == *id)
                        .ok_or(AiError::MissingVisibleCard(*id))?;
                    let destroys = u64::from(attacker.power) >= remaining_health(defender);
                    let survives = u64::from(defender.power) < remaining_health(attacker);
                    let target_value = u128::from(defender.power) + u128::from(defender.health);
                    let own_value = u128::from(attacker.power) + u128::from(attacker.health);
                    if destroys && survives {
                        (6, target_value, u128::from(u32::MAX - attacker.power))
                    } else if destroys && target_value > own_value {
                        (5, target_value - own_value, target_value)
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
        (Phase::Action, Command::ClaimInitiative) => (2, 0, 0),
        (Phase::Action, Command::Pass) => (1, 0, 0),
        _ => return Ok(None),
    };
    Ok(Some(Evaluation {
        score,
        key: command_key(command),
    }))
}

fn keep_hand(hand: &[(CardId, &CardDefinition)]) -> bool {
    hand.iter().any(|(_, card)| card.cost <= 2)
        && hand.iter().filter(|(_, card)| card.cost <= 4).count() >= 4
}

fn resource_value(
    hand: &[(CardId, &CardDefinition)],
    card: &CardDefinition,
    available: u128,
) -> u128 {
    let duplicates = hand.iter().filter(|(_, held)| held.id == card.id).count() as u128;
    let cannot_play_soon = u128::from(u128::from(card.cost) > available);
    (cannot_play_soon << 80)
        + (duplicates << 64)
        + (u128::from(card.cost) << 32)
        + u128::from(u32::MAX - card.power)
        + u128::from(u32::MAX - card.health)
}

fn remaining_health(unit: &UnitView) -> u64 {
    u64::from(unit.health).saturating_sub(unit.damage)
}

const fn public_player(
    view: &PlayerView,
    player: PlayerId,
) -> &low_kingdoms_rules::view::PublicPlayer {
    match player {
        PlayerId::Zero => &view.players[0],
        PlayerId::One => &view.players[1],
    }
}

const fn own_player(view: &PlayerView) -> &low_kingdoms_rules::view::PublicPlayer {
    public_player(view, view.player)
}

fn definition(catalog: &[CardDefinition], id: DefinitionId) -> Result<&CardDefinition, AiError> {
    catalog
        .iter()
        .find(|card| card.id == id)
        .ok_or(AiError::UnknownDefinition(id))
}

fn held_definition<'a>(
    hand: &[(CardId, &'a CardDefinition)],
    id: CardId,
) -> Result<&'a CardDefinition, AiError> {
    hand.iter()
        .find_map(|(card_id, definition)| (*card_id == id).then_some(*definition))
        .ok_or(AiError::MissingVisibleCard(id))
}

fn command_key(command: &Command) -> CommandKey {
    match command {
        Command::KeepHand => CommandKey::Keep,
        Command::MulliganHand => CommandKey::Mulligan,
        Command::ChooseInitialResources { cards } => CommandKey::Initial(cards.clone()),
        Command::ChooseRegroupResource { card } => CommandKey::Regroup(*card),
        Command::PlayUnit { card } => CommandKey::Play(*card),
        Command::Attack {
            attacker,
            target: AttackTarget::Unit(defender),
        } => CommandKey::AttackUnit(*attacker, *defender),
        Command::Attack {
            attacker,
            target: AttackTarget::Stronghold(player),
        } => CommandKey::AttackStronghold(*attacker, *player),
        Command::ClaimInitiative => CommandKey::Claim,
        Command::Pass => CommandKey::Pass,
    }
}

/// Unexpected integration inputs for deterministic AI; callers should report these defects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiError {
    /// No provided legal command matches the current public phase and useful priorities.
    NoLegalCommand,
    /// A visible hand references a definition missing from the public catalog.
    UnknownDefinition(DefinitionId),
    /// A supplied command references a card absent from the permitted view.
    MissingVisibleCard(CardId),
}

impl fmt::Display for AiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoLegalCommand => formatter.write_str("AI has no applicable legal command"),
            Self::UnknownDefinition(id) => {
                write!(formatter, "AI visible card definition {} is missing", id.0)
            }
            Self::MissingVisibleCard(id) => {
                write!(
                    formatter,
                    "AI command references absent visible card {}",
                    id.0
                )
            }
        }
    }
}

impl std::error::Error for AiError {}
