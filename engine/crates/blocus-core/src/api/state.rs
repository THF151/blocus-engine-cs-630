//! Public state-related API DTOs.

use crate::{
    BoardState, GameId, GameMode, PLAYER_COLOR_COUNT, PieceInventory, PlayerSlots, StateVersion,
    TurnOrder, TurnState, ZobristHash,
};

/// Current serialized state schema version.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StateSchemaVersion(u16);

impl StateSchemaVersion {
    /// Current state schema version.
    pub const CURRENT: Self = Self(1);

    /// Creates a schema version from a raw value.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the raw schema version.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

impl Default for StateSchemaVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl From<StateSchemaVersion> for u16 {
    fn from(value: StateSchemaVersion) -> Self {
        value.0
    }
}

/// Current game status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum GameStatus {
    /// Game is active.
    InProgress,
    /// Game has finished.
    Finished,
}

/// Scoring mode used for final scoring.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ScoringMode {
    /// Basic scoring: fewer remaining squares wins.
    Basic,
    /// Advanced scoring: remaining squares are negative, with completion
    /// bonuses.
    Advanced,
}

/// Public game-state DTO.
///
/// This is a value object. It deliberately stores compact domain primitives
/// instead of Python-facing wrapper objects.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GameState {
    /// State schema version.
    pub schema_version: StateSchemaVersion,
    /// Game identifier.
    pub game_id: GameId,
    /// Game mode.
    pub mode: GameMode,
    /// Scoring mode.
    pub scoring: ScoringMode,
    /// Game-specific turn order.
    pub turn_order: TurnOrder,
    /// Player/color assignment.
    pub player_slots: PlayerSlots,
    /// Board occupancy state.
    pub board: BoardState,
    /// Per-color inventories.
    pub inventories: [PieceInventory; PLAYER_COLOR_COUNT],
    /// Turn progression state.
    pub turn: TurnState,
    /// Game status.
    pub status: GameStatus,
    /// Monotonic state version.
    pub version: StateVersion,
    /// Semantic state hash placeholder.
    pub hash: ZobristHash,
}
