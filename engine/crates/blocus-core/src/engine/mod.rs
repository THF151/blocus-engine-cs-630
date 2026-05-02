//! Engine facade and state-transition orchestration.
//!
//! This module exposes the high-level domain API. Concrete move validation,
//! move generation, scoring, and hashing are delegated to focused modules as
//! they become available.

use crate::pieces::{PieceInventory, PieceRepository, standard_repository};
use crate::{
    BoardState, Command, DomainError, EngineError, GameConfig, GameResult, GameState, GameStatus,
    LegalMove, PLAYER_COLOR_COUNT, PlayerColor, PlayerId, ScoringMode, StateSchemaVersion,
    StateVersion, ZobristHash,
};

/// Pure Rust Blokus engine facade.
///
/// The facade owns no game state. It only holds a shared immutable reference to
/// the official precomputed piece repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BlocusEngine {
    pieces: &'static PieceRepository,
}

impl BlocusEngine {
    /// Creates a new stateless engine facade backed by the shared repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pieces: standard_repository(),
        }
    }

    /// Returns the immutable official piece repository used by this engine.
    #[must_use]
    pub const fn piece_repository(self) -> &'static PieceRepository {
        self.pieces
    }

    /// Initializes a new game from a validated configuration.
    #[must_use]
    pub fn initialize_game(&self, config: GameConfig) -> GameState {
        GameState {
            schema_version: StateSchemaVersion::CURRENT,
            game_id: config.game_id(),
            mode: config.mode(),
            scoring: config.scoring(),
            board: BoardState::EMPTY,
            turn_order: config.turn_order(),
            player_slots: config.player_slots(),
            inventories: [PieceInventory::EMPTY; PLAYER_COLOR_COUNT],
            turn: crate::TurnState::new(config.turn_order()),
            status: GameStatus::InProgress,
            version: StateVersion::INITIAL,
            hash: ZobristHash::ZERO,
        }
    }

    /// Applies a command to a game state.
    ///
    /// # Errors
    ///
    /// Currently returns [`EngineError::InvariantViolation`] because full command
    /// application has not been implemented yet.
    pub fn apply(&self, _state: &GameState, _command: Command) -> Result<GameResult, DomainError> {
        Err(EngineError::InvariantViolation.into())
    }

    /// Returns an iterator over valid moves.
    ///
    /// # Errors
    ///
    /// Currently returns [`EngineError::InvariantViolation`] because legal move
    /// generation has not been implemented yet.
    pub fn valid_moves_iter(
        &self,
        _state: &GameState,
        _player: PlayerId,
        _color: PlayerColor,
    ) -> Result<core::iter::Empty<LegalMove>, DomainError> {
        Err(EngineError::InvariantViolation.into())
    }

    /// Materializes all valid moves for a player/color.
    ///
    /// # Errors
    ///
    /// Currently returns [`EngineError::InvariantViolation`] because legal move
    /// generation has not been implemented yet.
    pub fn get_valid_moves(
        &self,
        _state: &GameState,
        _player: PlayerId,
        _color: PlayerColor,
    ) -> Result<Vec<LegalMove>, DomainError> {
        Err(EngineError::InvariantViolation.into())
    }

    /// Returns whether a player/color has any valid move.
    ///
    /// # Errors
    ///
    /// Currently returns [`EngineError::InvariantViolation`] because legal move
    /// generation has not been implemented yet.
    pub fn has_any_valid_move(
        &self,
        _state: &GameState,
        _player: PlayerId,
        _color: PlayerColor,
    ) -> Result<bool, DomainError> {
        Err(EngineError::InvariantViolation.into())
    }

    /// Scores a finished game.
    ///
    /// # Errors
    ///
    /// Currently returns [`EngineError::InvariantViolation`] because scoring has
    /// not been implemented yet.
    pub fn score_game(
        &self,
        _state: &GameState,
        _scoring: ScoringMode,
    ) -> Result<crate::ScoreBoard, DomainError> {
        Err(EngineError::InvariantViolation.into())
    }
}

impl Default for BlocusEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns true when the native engine crate is linked and callable.
#[must_use]
pub const fn engine_health() -> bool {
    true
}
