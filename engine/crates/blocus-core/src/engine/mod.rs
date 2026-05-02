//! Engine facade and state-transition orchestration.
//!
//! This module exposes the high-level domain API. Concrete move validation,
//! move generation, scoring, and hashing are delegated to focused modules.

use crate::api::state::LastPieceByColor;
use crate::pieces::{PieceInventory, PieceRepository};
use crate::{
    BoardMask, BoardState, Command, DomainError, DomainEvent, DomainEventKind, DomainResponse,
    DomainResponseKind, EngineError, GameConfig, GameResult, GameState, GameStatus, LegalMove,
    PLAYER_COLOR_COUNT, PassCommand, PlaceCommand, PlayerColor, PlayerId, RuleViolation,
    ScoringMode, StateSchemaVersion, StateVersion, standard_repository,
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
    #[must_use]
    pub fn initialize_game(&self, config: GameConfig) -> GameState {
        let mut state = GameState {
            schema_version: StateSchemaVersion::CURRENT,
            game_id: config.game_id(),
            mode: config.mode(),
            scoring: config.scoring(),
            board: BoardState::EMPTY,
            turn_order: config.turn_order(),
            player_slots: config.player_slots(),
            inventories: [PieceInventory::EMPTY; PLAYER_COLOR_COUNT],
            last_piece_by_color: LastPieceByColor::EMPTY,
            turn: crate::TurnState::new(config.turn_order()),
            status: GameStatus::InProgress,
            version: StateVersion::INITIAL,
            hash: crate::ZobristHash::ZERO,
        };

        state.hash = crate::compute_hash_full(&state);
        state
    }

    /// Applies a command to a game state.
    ///
    /// # Errors
    ///
    /// Returns a typed domain error if the command is invalid for the current
    /// state.
    pub fn apply(&self, state: &GameState, command: Command) -> Result<GameResult, DomainError> {
        validate_game_state(state)?;

        match command {
            Command::Place(command) => apply_place_command(state, command, self.piece_repository()),
            Command::Pass(command) => apply_pass_command(state, command, self.piece_repository()),
        }
    }

    /// Returns a lazy iterator over valid moves.
    ///
    /// The iterator is a snapshot: it copies the state-derived data it needs at
    /// construction time, so it does not borrow `state`.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::CorruptedState`] if `state` violates compact board
    /// invariants.
    pub fn valid_moves_iter(
        &self,
        state: &GameState,
        player: PlayerId,
        color: PlayerColor,
    ) -> Result<crate::movegen::LegalMoveIter, DomainError> {
        crate::movegen::legal_moves_iter(state, self.piece_repository(), player, color)
    }

    /// Materializes all valid moves for a player/color.
    ///
    /// # Errors
    ///
    /// Propagates move-iterator construction errors.
    pub fn get_valid_moves(
        &self,
        state: &GameState,
        player: PlayerId,
        color: PlayerColor,
    ) -> Result<Vec<LegalMove>, DomainError> {
        crate::movegen::get_valid_moves(state, self.piece_repository(), player, color)
    }

    /// Returns whether a player/color has any valid move.
    ///
    /// # Errors
    ///
    /// Propagates move-iterator construction errors.
    pub fn has_any_valid_move(
        &self,
        state: &GameState,
        player: PlayerId,
        color: PlayerColor,
    ) -> Result<bool, DomainError> {
        crate::movegen::has_any_valid_move(state, self.piece_repository(), player, color)
    }

    /// Scores a finished game.
    ///
    /// # Errors
    ///
    /// Returns [`RuleViolation::GameNotFinished`] when called before the game
    /// has finished.
    pub fn score_game(
        &self,
        state: &GameState,
        scoring: ScoringMode,
    ) -> Result<crate::ScoreBoard, DomainError> {
        validate_game_state(state)?;
        crate::scoring::score_game(state, self.piece_repository(), scoring)
    }
}

impl Default for BlocusEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates cheap structural invariants for a game state.
///
/// This function intentionally validates state shape, not full historical
/// reachability. It is suitable for public API boundaries and deserialization
/// boundaries.
///
/// # Errors
///
/// Returns [`EngineError::CorruptedState`] when board masks contain padding
/// bits, color masks overlap, or in-progress turn state has no active
/// controller.
pub fn validate_game_state(state: &GameState) -> Result<(), DomainError> {
    validate_board_invariants(state)?;
    validate_turn_invariants(state)?;

    Ok(())
}

fn validate_board_invariants(state: &GameState) -> Result<(), DomainError> {
    let mut occupied = BoardMask::EMPTY;

    for color in PlayerColor::ALL {
        let color_mask = state.board.occupied(color);

        if !color_mask.is_playable_subset() {
            return Err(EngineError::CorruptedState.into());
        }

        if occupied.intersects(color_mask) {
            return Err(EngineError::CorruptedState.into());
        }

        occupied = occupied.union(color_mask);
    }

    Ok(())
}

fn validate_turn_invariants(state: &GameState) -> Result<(), DomainError> {
    if state.status == GameStatus::InProgress {
        if state.turn.is_passed(state.turn.current_color()) {
            return Err(EngineError::CorruptedState.into());
        }

        if state.turn.active_controller(state.player_slots).is_none() {
            return Err(EngineError::CorruptedState.into());
        }
    }

    Ok(())
}

fn apply_place_command(
    state: &GameState,
    command: PlaceCommand,
    repository: &'static PieceRepository,
) -> Result<GameResult, DomainError> {
    let placement = crate::validate_place_command(state, command, repository)?;

    let mut next_state = state.clone();

    next_state.board.place_mask(command.color, placement.mask());
    next_state.inventories[command.color.index()].mark_used(command.piece_id);
    next_state
        .last_piece_by_color
        .set(command.color, command.piece_id);

    let turn_advanced = next_state
        .turn
        .advance(next_state.turn_order, next_state.player_slots)
        .is_some();

    let is_finished = finalize_after_turn_resolution(&mut next_state, turn_advanced, repository);

    let mut events = Vec::with_capacity(2);
    events.push(DomainEvent {
        kind: DomainEventKind::MoveApplied,
        game_id: next_state.game_id,
        version: next_state.version,
    });

    if is_finished {
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
            kind: if is_finished {
                DomainResponseKind::GameFinished
            } else {
                DomainResponseKind::MoveApplied
            },
            message: if is_finished {
                "game finished".to_owned()
            } else {
                "move applied".to_owned()
            },
        },
    })
}

fn apply_pass_command(
    state: &GameState,
    command: PassCommand,
    repository: &'static PieceRepository,
) -> Result<GameResult, DomainError> {
    validate_pass_command(state, command, repository)?;

    let mut next_state = state.clone();
    next_state.turn.mark_passed(command.color);

    let turn_advanced = next_state
        .turn
        .advance(next_state.turn_order, next_state.player_slots)
        .is_some();

    let is_finished = finalize_after_turn_resolution(&mut next_state, turn_advanced, repository);

    let mut events = Vec::with_capacity(2);
    events.push(DomainEvent {
        kind: DomainEventKind::PlayerPassed,
        game_id: next_state.game_id,
        version: next_state.version,
    });

    if is_finished {
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
            kind: if is_finished {
                DomainResponseKind::GameFinished
            } else {
                DomainResponseKind::PlayerPassed
            },
            message: if is_finished {
                "game finished".to_owned()
            } else {
                "player passed".to_owned()
            },
        },
    })
}

fn finalize_after_turn_resolution(
    next_state: &mut GameState,
    turn_advanced: bool,
    repository: &'static PieceRepository,
) -> bool {
    if !turn_advanced || !any_unpassed_color_has_valid_move(next_state, repository) {
        next_state.status = GameStatus::Finished;
    }

    next_state.version = next_state.version.saturating_next();
    next_state.hash = crate::compute_hash_full(next_state);

    next_state.status == GameStatus::Finished
}

fn validate_pass_command(
    state: &GameState,
    command: PassCommand,
    repository: &'static PieceRepository,
) -> Result<(), DomainError> {
    if state.status != GameStatus::InProgress {
        return Err(RuleViolation::GameAlreadyFinished.into());
    }

    if command.game_id != state.game_id {
        return Err(crate::InputError::GameIdMismatch.into());
    }

    if state.turn.current_color() != command.color {
        return Err(RuleViolation::WrongPlayerTurn.into());
    }

    if !state
        .turn
        .is_active_controller(state.player_slots, command.player_id)
    {
        return Err(RuleViolation::PlayerDoesNotControlColor.into());
    }

    if crate::movegen::has_any_valid_move(state, repository, command.player_id, command.color)? {
        return Err(RuleViolation::PassNotAllowedBecauseMoveExists.into());
    }

    Ok(())
}

fn any_unpassed_color_has_valid_move(
    state: &GameState,
    repository: &'static PieceRepository,
) -> bool {
    for color in PlayerColor::ALL {
        if state.turn.is_passed(color) {
            continue;
        }

        let probe_turn = crate::TurnState::from_parts(
            color,
            state.turn.passed_mask(),
            state.turn.shared_color_turn_index(),
        );

        let Some(player_id) = probe_turn.current_player(state.player_slots) else {
            continue;
        };

        let mut probe = state.clone();
        probe.turn = probe_turn;

        if crate::movegen::has_any_valid_move(&probe, repository, player_id, color).unwrap_or(false)
        {
            return true;
        }
    }

    false
}

/// Returns true when the native engine crate is linked and callable.
#[must_use]
pub const fn engine_health() -> bool {
    true
}
