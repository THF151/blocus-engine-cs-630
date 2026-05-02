//! Public API contract types.

pub mod command;
pub mod event;
pub mod result;
pub mod score;
pub mod state;

pub use command::{Command, PassCommand, PlaceCommand};
pub use event::{DomainEvent, DomainEventKind, DomainResponse, DomainResponseKind};
pub use result::GameResult;
pub use score::{LegalMove, ScoreBoard, ScoreEntry};
pub use state::{GameState, GameStatus, LastPieceByColor, ScoringMode, StateSchemaVersion};
