//! Domain event and response DTOs.

use crate::{GameId, StateVersion};

/// Domain event kind emitted by successful state transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DomainEventKind {
    /// A piece was placed.
    MoveApplied,
    /// A player/color passed.
    PlayerPassed,
    /// The active turn advanced.
    TurnAdvanced,
    /// The game finished.
    GameFinished,
}

impl DomainEventKind {
    /// Returns the stable lowercase API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MoveApplied => "move_applied",
            Self::PlayerPassed => "player_passed",
            Self::TurnAdvanced => "turn_advanced",
            Self::GameFinished => "game_finished",
        }
    }
}

/// Pure domain event returned by the engine.
///
/// Events are data only.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DomainEvent {
    /// Event kind.
    pub kind: DomainEventKind,
    /// Game identifier.
    pub game_id: GameId,
    /// State version associated with this event.
    pub version: StateVersion,
}

/// Engine response summary kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DomainResponseKind {
    /// A move was applied.
    MoveApplied,
    /// A pass was applied.
    PlayerPassed,
    /// The game finished.
    GameFinished,
}

impl DomainResponseKind {
    /// Returns the stable lowercase API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MoveApplied => "move_applied",
            Self::PlayerPassed => "player_passed",
            Self::GameFinished => "game_finished",
        }
    }
}

/// Human-readable response summary.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DomainResponse {
    /// Response kind.
    pub kind: DomainResponseKind,
    /// Human-readable response message.
    pub message: String,
}
