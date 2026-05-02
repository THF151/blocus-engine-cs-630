#![allow(clippy::trivially_copy_pass_by_ref, clippy::unused_self)]

use crate::command::{PyPassCommand, PyPlaceCommand};
use crate::config::GameConfig;
use crate::conversion::{map_domain_error, parse_player_id};
use crate::errors::input_error;
use crate::pieces::Piece;
use crate::result::{GameResult, LegalMove, ScoreBoard};
use crate::state::GameState;
use crate::types::{PlayerColor, ScoringMode};
use pyo3::prelude::*;

#[pyclass(name = "BlocusEngine", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct BlocusEngine {
    inner: blocus_core::BlocusEngine,
}

#[pymethods]
impl BlocusEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: blocus_core::BlocusEngine::new(),
        }
    }

    fn initialize_game(&self, config: &GameConfig) -> GameState {
        GameState::from_core(self.inner.initialize_game(config.as_core()))
    }

    fn apply(
        &self,
        state: &GameState,
        command: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> PyResult<GameResult> {
        let command = parse_command(command)?;
        self.inner
            .apply(state.as_core(), command)
            .map(GameResult::from_core)
            .map_err(map_domain_error)
    }

    fn pieces(&self) -> Vec<Piece> {
        self.inner
            .piece_repository()
            .pieces()
            .iter()
            .copied()
            .map(Piece::from_core)
            .collect()
    }

    fn piece(&self, py: Python<'_>, piece_id: u8) -> PyResult<Piece> {
        let piece_id = parse_piece_id(py, piece_id)?;

        Ok(Piece::from_core(
            *self.inner.piece_repository().piece(piece_id),
        ))
    }

    fn get_valid_moves(
        &self,
        state: &GameState,
        player_id: &str,
        color: &PlayerColor,
    ) -> PyResult<Vec<LegalMove>> {
        let player_id = parse_player_id(player_id, "player_id")?;

        self.inner
            .get_valid_moves(state.as_core(), player_id, color.as_core())
            .map(|moves| moves.into_iter().map(LegalMove::from_core).collect())
            .map_err(map_domain_error)
    }

    fn get_valid_moves_for_piece(
        &self,
        py: Python<'_>,
        state: &GameState,
        player_id: &str,
        color: &PlayerColor,
        piece_id: u8,
    ) -> PyResult<Vec<LegalMove>> {
        let player_id = parse_player_id(player_id, "player_id")?;
        let piece_id = parse_piece_id(py, piece_id)?;

        self.inner
            .get_valid_moves_for_piece(state.as_core(), player_id, color.as_core(), piece_id)
            .map(|moves| moves.into_iter().map(LegalMove::from_core).collect())
            .map_err(map_domain_error)
    }

    fn has_any_valid_move(
        &self,
        state: &GameState,
        player_id: &str,
        color: &PlayerColor,
    ) -> PyResult<bool> {
        let player_id = parse_player_id(player_id, "player_id")?;

        self.inner
            .has_any_valid_move(state.as_core(), player_id, color.as_core())
            .map_err(map_domain_error)
    }

    fn has_any_valid_move_for_piece(
        &self,
        py: Python<'_>,
        state: &GameState,
        player_id: &str,
        color: &PlayerColor,
        piece_id: u8,
    ) -> PyResult<bool> {
        let player_id = parse_player_id(player_id, "player_id")?;
        let piece_id = parse_piece_id(py, piece_id)?;

        self.inner
            .has_any_valid_move_for_piece(state.as_core(), player_id, color.as_core(), piece_id)
            .map_err(map_domain_error)
    }

    fn score_game(&self, state: &GameState, scoring: &ScoringMode) -> PyResult<ScoreBoard> {
        self.inner
            .score_game(state.as_core(), scoring.as_core())
            .map(ScoreBoard::from_core)
            .map_err(map_domain_error)
    }

    fn __repr__(&self) -> &'static str {
        "BlocusEngine()"
    }
}

fn parse_command(command: &pyo3::Bound<'_, pyo3::PyAny>) -> PyResult<blocus_core::Command> {
    if let Ok(place) = command.extract::<PyPlaceCommand>() {
        return Ok(blocus_core::Command::Place(place.as_core()));
    }

    if let Ok(pass) = command.extract::<PyPassCommand>() {
        return Ok(blocus_core::Command::Pass(pass.as_core()));
    }

    Err(map_domain_error(
        blocus_core::EngineError::InvariantViolation.into(),
    ))
}

fn parse_piece_id(py: Python<'_>, value: u8) -> PyResult<blocus_core::PieceId> {
    blocus_core::PieceId::try_new(value)
        .map_err(|_| input_error(py, blocus_core::InputError::UnknownPiece))
}
