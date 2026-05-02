//! Engine facade and state-transition orchestration.
//!
//! This module exposes the high-level domain API. Concrete move validation,
//! move generation, scoring, and hashing are delegated to focused modules as
//! they become available.

use crate::pieces::{PieceInventory, PieceRepository};
use crate::{
    BoardState, Command, DomainError, DomainEvent, DomainEventKind, DomainResponse,
    DomainResponseKind, EngineError, GameConfig, GameResult, GameState, GameStatus, LegalMove,
    PLAYER_COLOR_COUNT, PlaceCommand, PlayerColor, PlayerId, ScoringMode, StateSchemaVersion,
    StateVersion, ZobristHash, standard_repository,
};

/// Pure Rust Blocus engine facade.
///
/// The facade is intentionally small. It owns no game state and has no
/// dependency on Python, storage, networking, or AI crates. It only holds a
/// reference to the immutable static piece repository so all engines share the
/// same precomputed canonical pieces and orientations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BlocusEngine {
    pieces: &'static PieceRepository,
}

impl BlocusEngine {
    /// Creates a new stateless engine facade backed by the standard immutable
    /// piece repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pieces: standard_repository(),
        }
    }

    /// Returns the shared immutable canonical piece repository.
    #[must_use]
    pub const fn piece_repository(self) -> &'static PieceRepository {
        self.pieces
    }

    /// Initializes a new game from a validated configuration.
    ///
    /// `GameConfig` is already a validated value object in the current model,
    /// so initialization only assembles the first immutable state snapshot.
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
    /// Returns a typed domain error if the command is invalid for the current
    /// state or if the command kind is not implemented yet.
    pub fn apply(&self, state: &GameState, command: Command) -> Result<GameResult, DomainError> {
        match command {
            Command::Place(command) => apply_place_command(state, command, self.piece_repository()),
            Command::Pass(_command) => Err(EngineError::InvariantViolation.into()),
        }
    }

    /// Returns an iterator over valid moves.
    ///
    /// This contract method exists before move generation is implemented. The
    /// concrete iterator type will be introduced with the move-generation
    /// module.
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

fn apply_place_command(
    state: &GameState,
    command: PlaceCommand,
    repository: &PieceRepository,
) -> Result<GameResult, DomainError> {
    let placement = crate::validate_place_command(state, command, repository)?;

    let mut next_state = state.clone();

    next_state.board.place_mask(command.color, placement.mask());
    next_state.inventories[command.color.index()].mark_used(command.piece_id);

    if next_state
        .turn
        .advance(next_state.turn_order, next_state.player_slots)
        .is_none()
    {
        next_state.status = GameStatus::Finished;
    }

    next_state.version = next_state.version.saturating_next();
    next_state.hash = ZobristHash::ZERO;

    let mut events = Vec::with_capacity(2);
    events.push(DomainEvent {
        kind: DomainEventKind::MoveApplied,
        game_id: next_state.game_id,
        version: next_state.version,
    });

    if next_state.status == GameStatus::Finished {
        events.push(DomainEvent {
            kind: DomainEventKind::GameFinished,
            game_id: next_state.game_id,
            version: next_state.version,
        });
    } else {
        events.push(DomainEvent {
            kind: DomainEventKind::TurnAdvanced,
            game_id: next_state.game_id,
            version: next_state.version,
        });
    }

    Ok(GameResult {
        next_state,
        events,
        response: DomainResponse {
            kind: DomainResponseKind::MoveApplied,
            message: "move applied".to_owned(),
        },
    })
}

/// Returns true when the native engine crate is linked and callable.
///
/// This remains useful as a deployment/linkage smoke check even after gameplay
/// APIs exist.
#[must_use]
pub const fn engine_health() -> bool {
    true
}
