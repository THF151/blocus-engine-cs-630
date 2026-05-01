//! Core domain engine for Blocus.

pub mod board;
pub mod color;
pub mod engine;
pub mod error;
pub mod ids;

pub use board::{
    BOARD_BITS, BOARD_LANES, BOARD_SIZE, BoardIndex, BoardMask, BoardState, PLAYABLE_CELLS,
    PLAYABLE_MASK, ROW_PADDING_BITS, ROW_STRIDE,
};

pub use color::{PLAYER_COLOR_COUNT, PlayerColor, TurnOrder, TurnOrderPolicy};
pub use engine::engine_health;
pub use error::{DomainError, EngineError, InputError, RuleViolation};
pub use ids::{
    CommandId, GameId, MAX_ORIENTATION_COUNT, OrientationId, PIECE_COUNT, PieceId, PlayerId,
    SmallIdError, StateVersion, ZobristHash,
};
