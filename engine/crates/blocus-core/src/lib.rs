//! Core domain engine for Blocus.
//!
//! `blocus-core` contains the pure Rust domain model and engine foundation. It
//! must not depend on `Python`, `FastAPI`, `Redis`, `WebSockets`, `Flutter`, or
//! AI crates.

pub mod api;
pub mod board;
pub mod color;
pub mod config;
pub mod engine;
pub mod error;
pub mod ids;
pub mod pieces;

pub use api::{
    Command, DomainEvent, DomainEventKind, DomainResponse, DomainResponseKind, GameResult,
    GameState, GameStatus, LegalMove, PassCommand, PlaceCommand, ScoreBoard, ScoreEntry,
    ScoringMode, StateSchemaVersion,
};
pub use board::{
    BOARD_BITS, BOARD_LANES, BOARD_SIZE, BoardIndex, BoardMask, BoardState, PLAYABLE_CELLS,
    PLAYABLE_MASK, ROW_PADDING_BITS, ROW_STRIDE,
};
pub use color::{PLAYER_COLOR_COUNT, PlayerColor, TurnOrder, TurnOrderPolicy};
pub use config::{GameConfig, GameMode, PlayerSlots, SharedColorTurn, TurnState};
pub use engine::engine_health;
pub use error::{DomainError, EngineError, InputError, RuleViolation};
pub use ids::{
    CommandId, GameId, MAX_ORIENTATION_COUNT, OrientationId, PIECE_COUNT, PieceId, PlayerId,
    SmallIdError, StateVersion, ZobristHash,
};
pub use pieces::{ALL_PIECES_MASK, PieceInventory};
