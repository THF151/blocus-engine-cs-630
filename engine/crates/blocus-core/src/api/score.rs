//! Legal-move and scoring DTOs.

use crate::{BoardIndex, OrientationId, PieceId, PlayerId, ScoringMode};

/// Legal move returned by the engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LegalMove {
    /// Piece to place.
    pub piece_id: PieceId,
    /// Precomputed orientation to place.
    pub orientation_id: OrientationId,
    /// Placement anchor.
    pub anchor: BoardIndex,
    /// Immediate square-count delta.
    pub score_delta: u8,
}

/// One scoreboard entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ScoreEntry {
    /// Player identifier.
    pub player_id: PlayerId,
    /// Final score.
    pub score: i16,
}

/// Final scoreboard.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ScoreBoard {
    /// Scoring mode used.
    pub scoring: ScoringMode,
    /// Score entries.
    pub entries: Vec<ScoreEntry>,
}
