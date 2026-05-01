//! Core domain engine for Blocus.

pub mod engine;
pub mod error;
pub mod ids;

pub use engine::engine_health;
pub use error::{DomainError, EngineError, InputError, RuleViolation};
pub use ids::{
    CommandId, GameId, MAX_ORIENTATION_COUNT, OrientationId, PIECE_COUNT, PieceId, PlayerId,
    SmallIdError, StateVersion, ZobristHash,
};
