//! Authoritative human-versus-AI sessions and privileged, versioned local-save data.
//!
//! Presentation receives only player-zero views and filtered events. The complete rules snapshot
//! and command log are available exclusively through the privileged save/restore boundary.

use std::fmt;

use low_kingdoms_rules::command::{Command, LoggedCommand};
use low_kingdoms_rules::engine::Game;
use low_kingdoms_rules::error::Error as RulesError;
use low_kingdoms_rules::event::Event;
use low_kingdoms_rules::model::{Phase, PlayerId};
use low_kingdoms_rules::view::PlayerView;
use serde::{Deserialize, Serialize};

use crate::ai::{self, AiError};
use crate::content::{CardDefinition, Content};

const HUMAN: PlayerId = PlayerId::Zero;
const OPPONENT: PlayerId = PlayerId::One;
const SAVE_VERSION: u32 = 1;
const AI_PROFILE_VERSION: u32 = 1;
const MAX_SAVE_BYTES: usize = 2 * 1024 * 1024;
// Twenty-card decks bound rounds through mandatory exhaustion damage. This protective limit is
// comfortably above a complete match's deployments, attacks, resource decisions, and passes.
const MAX_COMMANDS: usize = 4096;

/// Maximum retained human-visible presentation events, including initialization.
///
/// Older events are discarded from the presentation history only. The privileged command log
/// retains every successful command for exact save validation and deterministic continuation.
pub const HISTORY_LIMIT: usize = 256;

/// A session initialization, command, AI, or privileged save failure.
#[derive(Debug)]
pub enum SessionError {
    /// The authoritative rules engine rejected the operation without mutation.
    Rules(RulesError),
    /// The deterministic opponent could not choose a command; no substitute is applied.
    Ai(AiError),
    /// No opponent command is currently available.
    NoAiDecision,
    /// A session counter cannot advance or a protective save bound was exceeded.
    CounterLimit,
    /// Save JSON is malformed, has unknown fields, or exceeds the size bound.
    SaveFormat,
    /// Save format version is not supported.
    SaveVersion,
    /// Definitions, display names, decks, or strongholds differ from the saved content.
    SaveContent,
    /// Saved human/opponent identities do not match this single-player application.
    SaveRoles,
    /// Saved AI profile or deterministic state is incompatible.
    SaveAiProfile,
    /// Snapshot, command log, revision, phase, or filtered history are inconsistent.
    SaveInvariant,
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Self::Rules(_) => "The battle command was rejected.",
            Self::Ai(_) => "The opponent could not choose an action.",
            Self::NoAiDecision => "The opponent has no pending action.",
            Self::CounterLimit => "The battle exceeds the supported save limits.",
            Self::SaveFormat => "The battle save is damaged or has an unsupported structure.",
            Self::SaveVersion => "The battle save uses an incompatible format version.",
            Self::SaveContent => "The battle save uses different card or deck content.",
            Self::SaveRoles => "The battle save has incompatible player assignments.",
            Self::SaveAiProfile => "The battle save uses an incompatible opponent profile.",
            Self::SaveInvariant => "The battle save does not describe a consistent match.",
        };
        formatter.write_str(description)
    }
}

impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rules(error) => Some(error),
            Self::Ai(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RulesError> for SessionError {
    fn from(error: RulesError) -> Self {
        Self::Rules(error)
    }
}

/// One authoritative match, owned by the client and independent of Bevy or platform storage.
///
/// Human identity is always player zero. AI identity is always player one. The AI profile is
/// stateless: each action depends only on its permitted view, legal commands, and static content.
/// No timer, background task, queued action, selection, or stale AI decision enters save data.
/// Each successful command increments [`Self::revision`], including a buffered setup decision
/// that emits no public events. Invalid commands leave the entire session unchanged. Local saves
/// are limited to 2 MiB and 4,096 commands, above the fixed prototype decks' complete-match needs.
pub struct Session {
    game: Game,
    signature: String,
    seed: u64,
    revision: u64,
    commands: Vec<LoggedCommand>,
    history: Vec<Event>,
}

impl Session {
    /// Initializes a fresh match from validated prototype content and an explicit seed.
    ///
    /// Initialization events are filtered for the human and recorded immediately. Seed selection
    /// and local storage belong to the platform edge, not this deterministic session.
    ///
    /// # Errors
    /// Returns the rules engine's initialization failure if supplied content is incompatible.
    pub fn new(content: &Content, seed: u64) -> Result<Self, SessionError> {
        let definitions = content
            .cards()
            .iter()
            .map(CardDefinition::unit_definition)
            .collect();
        let (game, events) = Game::new(content.config(seed), definitions)?;
        let mut history = Vec::new();
        append_history(&mut history, &events.for_player(HUMAN));
        Ok(Self {
            game,
            signature: content.signature().to_owned(),
            seed,
            revision: 0,
            commands: Vec::new(),
            history,
        })
    }

    /// Restores a privileged save against the exact current content and AI profile.
    ///
    /// Restore validates the engine snapshot, replays every recorded successful command, checks
    /// AI choices against the deterministic profile, and compares the resulting authoritative
    /// state, revision, phase, and bounded filtered history. Pending decisions are preserved;
    /// scheduling can inspect [`Self::needs_ai`] without duplicating a completed action. Finished
    /// states also validate and restore; callers use [`Self::is_finished`] to clear the active
    /// slot and refuse resume rather than misclassifying a failed cleanup as a corrupt save.
    ///
    /// # Errors
    /// Rejects malformed/oversized data, unknown fields, unsupported versions or roles, different
    /// content, invalid snapshots/logs, or inconsistent history/metadata.
    pub fn restore(content: &Content, json: &str) -> Result<Self, SessionError> {
        if json.len() > MAX_SAVE_BYTES {
            return Err(SessionError::SaveFormat);
        }
        let saved: Save = serde_json::from_str(json).map_err(|_| SessionError::SaveFormat)?;
        saved.validate_header(content)?;
        let restored = Game::restore(
            &saved.snapshot,
            content
                .cards()
                .iter()
                .map(CardDefinition::unit_definition)
                .collect(),
        )?;
        let mut replayed = Self::new(content, saved.seed)?;
        for entry in &saved.commands {
            if entry.player == OPPONENT {
                let chosen = ai::choose(
                    &replayed.game.view_for(OPPONENT),
                    &replayed.game.legal_commands_for(OPPONENT),
                    content.cards(),
                )
                .map_err(|_| SessionError::SaveInvariant)?;
                if chosen != entry.command {
                    return Err(SessionError::SaveInvariant);
                }
            }
            replayed
                .apply(entry.player, &entry.command)
                .map_err(|_| SessionError::SaveInvariant)?;
        }
        if restored.snapshot()? != replayed.game.snapshot()?
            || replayed.revision != saved.revision
            || replayed.history != saved.history
            || restored.view_for(HUMAN).phase != saved.phase
        {
            return Err(SessionError::SaveInvariant);
        }
        replayed.game = restored;
        Ok(replayed)
    }

    /// Returns only the human player's permitted state, never an opposing private view.
    #[must_use]
    pub fn view(&self) -> PlayerView {
        self.game.view_for(HUMAN)
    }

    /// Lists the human's currently legal commands using authoritative engine validation.
    #[must_use]
    pub fn legal_commands(&self) -> Vec<Command> {
        self.game.legal_commands_for(HUMAN)
    }

    /// Whether the opponent has a legal decision/action to execute now.
    ///
    /// This reveals eligibility only. No private decision, command, hand, or resource is exposed.
    #[must_use]
    pub fn needs_ai(&self) -> bool {
        !self.game.legal_commands_for(OPPONENT).is_empty()
    }

    /// Validates and applies one human command, returning human-filtered transition events.
    ///
    /// # Errors
    /// Returns a rules rejection or representational/save-bound limit without session mutation.
    pub fn apply_human(&mut self, command: &Command) -> Result<Vec<Event>, SessionError> {
        self.apply(HUMAN, command)
    }

    /// Chooses and executes exactly one currently available deterministic opponent command.
    ///
    /// The AI receives only its own player-specific view and the engine's legal actions. Choice
    /// failures or rejected commands are returned directly; no fallback command hides a defect.
    ///
    /// # Errors
    /// Rejects changed content, an unavailable opponent action, AI failure, counter limits, or
    /// an engine command rejection. Every failure preserves the last valid session completely.
    pub fn step_ai(&mut self, content: &Content) -> Result<Vec<Event>, SessionError> {
        if content.signature() != self.signature {
            return Err(SessionError::SaveContent);
        }
        let legal = self.game.legal_commands_for(OPPONENT);
        if legal.is_empty() {
            return Err(SessionError::NoAiDecision);
        }
        let command = ai::choose(&self.game.view_for(OPPONENT), &legal, content.cards())
            .map_err(SessionError::Ai)?;
        self.apply(OPPONENT, &command)
    }

    /// Returns the chronological, bounded human-visible event history.
    #[must_use]
    pub fn history(&self) -> &[Event] {
        &self.history
    }

    /// Number of successfully completed commands, including private buffered decisions.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the match's privileged platform-control seed, not a player-view field.
    ///
    /// Never expose this value through production UI or browser globals: with static content it
    /// can reconstruct private shuffles. It is used for local save/control diagnostics only.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Whether the rules engine determined a terminal outcome.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.view().outcome.is_some()
    }

    /// Encodes complete privileged authoritative save data without performing platform I/O.
    ///
    /// **Privileged:** the snapshot and command log include hidden cards and pending decisions.
    /// Do not display this JSON, attach it to public event history, or expose it in browser globals.
    /// Completed sessions can be encoded as a fallback when removing the active slot fails.
    /// Callers clear the active save and refuse resume when [`Self::is_finished`] becomes true.
    ///
    /// # Errors
    /// Returns a snapshot/serialization failure or an encoded size exceeding the supported bound.
    pub fn save(&self) -> Result<String, SessionError> {
        let encoded = serde_json::to_string(&SaveRef {
            version: SAVE_VERSION,
            content_signature: &self.signature,
            human: HUMAN,
            opponent: OPPONENT,
            ai_profile_version: AI_PROFILE_VERSION,
            ai_state: AiState::Stateless,
            seed: self.seed,
            revision: self.revision,
            phase: self.view().phase,
            snapshot: self.game.snapshot()?,
            commands: &self.commands,
            history: &self.history,
        })
        .map_err(|_| SessionError::SaveFormat)?;
        if encoded.len() > MAX_SAVE_BYTES {
            return Err(SessionError::CounterLimit);
        }
        Ok(encoded)
    }

    fn apply(&mut self, player: PlayerId, command: &Command) -> Result<Vec<Event>, SessionError> {
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(SessionError::CounterLimit)?;
        if self.commands.len() >= MAX_COMMANDS {
            return Err(SessionError::CounterLimit);
        }
        let events = self.game.apply(player, command)?.for_player(HUMAN);
        self.revision = revision;
        self.commands.push(LoggedCommand {
            player,
            command: command.clone(),
        });
        append_history(&mut self.history, &events);
        Ok(events)
    }
}

fn append_history(history: &mut Vec<Event>, events: &[Event]) {
    history.extend_from_slice(events);
    if history.len() > HISTORY_LIMIT {
        history.drain(..history.len() - HISTORY_LIMIT);
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
enum AiState {
    Stateless,
}

#[derive(Serialize)]
struct SaveRef<'a> {
    version: u32,
    content_signature: &'a str,
    human: PlayerId,
    opponent: PlayerId,
    ai_profile_version: u32,
    ai_state: AiState,
    seed: u64,
    revision: u64,
    phase: Phase,
    snapshot: String,
    commands: &'a [LoggedCommand],
    history: &'a [Event],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    version: u32,
    content_signature: String,
    human: PlayerId,
    opponent: PlayerId,
    ai_profile_version: u32,
    ai_state: AiState,
    seed: u64,
    revision: u64,
    phase: Phase,
    snapshot: String,
    commands: Vec<LoggedCommand>,
    history: Vec<Event>,
}

impl Save {
    fn validate_header(&self, content: &Content) -> Result<(), SessionError> {
        if self.version != SAVE_VERSION {
            return Err(SessionError::SaveVersion);
        }
        if self.content_signature != content.signature() {
            return Err(SessionError::SaveContent);
        }
        if self.human != HUMAN || self.opponent != OPPONENT {
            return Err(SessionError::SaveRoles);
        }
        if self.ai_profile_version != AI_PROFILE_VERSION
            || !matches!(self.ai_state, AiState::Stateless)
        {
            return Err(SessionError::SaveAiProfile);
        }
        if self.commands.len() > MAX_COMMANDS
            || self.history.len() > HISTORY_LIMIT
            || usize::try_from(self.revision).ok() != Some(self.commands.len())
        {
            return Err(SessionError::SaveInvariant);
        }
        Ok(())
    }
}
