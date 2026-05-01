#![allow(clippy::trivially_copy_pass_by_ref, clippy::unused_self)]
use crate::config::GameConfig;
use crate::conversion::map_domain_error;
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

    fn apply(&self, state: &GameState, command: &pyo3::Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        let _ = state;
        let _ = command;
        Err(map_domain_error(
            blocus_core::EngineError::InvariantViolation.into(),
        ))
    }

    fn get_valid_moves(
        &self,
        state: &GameState,
        player_id: &str,
        color: &PlayerColor,
    ) -> PyResult<Vec<()>> {
        let _ = state;
        let _ = player_id;
        let _ = color;
        Err(map_domain_error(
            blocus_core::EngineError::InvariantViolation.into(),
        ))
    }

    fn has_any_valid_move(
        &self,
        state: &GameState,
        player_id: &str,
        color: &PlayerColor,
    ) -> PyResult<bool> {
        let _ = state;
        let _ = player_id;
        let _ = color;
        Err(map_domain_error(
            blocus_core::EngineError::InvariantViolation.into(),
        ))
    }

    fn score_game(&self, state: &GameState, scoring: &ScoringMode) -> PyResult<()> {
        let _ = state;
        let _ = scoring;
        Err(map_domain_error(
            blocus_core::EngineError::InvariantViolation.into(),
        ))
    }

    fn __repr__(&self) -> &'static str {
        "BlocusEngine()"
    }
}
