//! Public state-related API DTOs.

use crate::{BoardState, StateVersion, ZobristHash};

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

/// Minimal public game-state DTO for the current contract slice.
///
/// This shape is intentionally still small.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GameState {
    /// State schema version.
    pub schema_version: StateSchemaVersion,
    /// Board occupancy state.
    pub board: BoardState,
    /// Game status.
    pub status: GameStatus,
    /// Monotonic state version.
    pub version: StateVersion,
    /// Semantic state hash placeholder.
    pub hash: ZobristHash,
}
