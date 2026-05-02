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
pub mod hash;
pub mod ids;
pub mod movegen;
pub mod pieces;
pub mod rules;
pub mod scoring;

pub use api::{
    Command, DomainEvent, DomainEventKind, DomainResponse, DomainResponseKind, GameResult,
    GameState, GameStatus, LastPieceByColor, LegalMove, PassCommand, PlaceCommand, ScoreBoard,
    ScoreEntry, ScoringMode, StateSchemaVersion,
};
pub use board::{
    BOARD_BITS, BOARD_LANES, BOARD_SIZE, BoardIndex, BoardMask, BoardState, PLAYABLE_CELLS,
    PLAYABLE_MASK, ROW_PADDING_BITS, ROW_STRIDE,
};
pub use color::{PLAYER_COLOR_COUNT, PlayerColor, TurnOrder, TurnOrderPolicy};
pub use config::{GameConfig, GameMode, PlayerSlots, SharedColorTurn, TurnState};
pub use engine::{BlocusEngine, engine_health, validate_game_state};
pub use error::{DomainError, EngineError, InputError, RuleViolation};
pub use hash::{board_cell_hash, compute_hash_full, inventory_piece_hash};
pub use ids::{
    CommandId, GameId, MAX_ORIENTATION_COUNT, OrientationId, PIECE_COUNT, PieceId, PlayerId,
    SmallIdError, StateVersion, ZobristHash,
};
pub use movegen::LegalMoveIter;
pub use pieces::{
    ALL_PIECES_MASK, CanonicalPiece, Flip, MAX_SHAPE_CELLS, MAX_SHAPE_EXTENT,
    MAX_UNIQUE_ORIENTATIONS, PieceInventory, PieceOrientation, PieceRepository, Rotation,
    ShapeBitmap, standard_piece, standard_pieces, standard_repository,
};
pub use rules::{Placement, build_placement, validate_place_command};
pub use scoring::score_game;
