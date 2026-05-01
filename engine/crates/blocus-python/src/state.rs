use crate::config::{GameMode, PlayerSlots};
use crate::types::{GameStatus, PlayerColor, ScoringMode};
use pyo3::prelude::*;

#[pyclass(name = "GameState", frozen, skip_from_py_object)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GameState {
    inner: blocus_core::GameState,
}

impl GameState {
    pub const fn from_core(inner: blocus_core::GameState) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> &blocus_core::GameState {
        &self.inner
    }
}

#[pymethods]
impl GameState {
    #[getter]
    fn schema_version(&self) -> u16 {
        self.inner.schema_version.as_u16()
    }

    #[getter]
    fn game_id(&self) -> String {
        self.inner.game_id.to_string()
    }

    #[getter]
    fn mode(&self) -> GameMode {
        GameMode::from_core(self.inner.mode)
    }

    #[getter]
    fn scoring(&self) -> ScoringMode {
        ScoringMode::from_core(self.inner.scoring)
    }

    #[getter]
    fn status(&self) -> GameStatus {
        GameStatus::from_core(self.inner.status)
    }

    #[getter]
    fn version(&self) -> u64 {
        self.inner.version.as_u64()
    }

    #[getter]
    fn hash(&self) -> u64 {
        self.inner.hash.as_u64()
    }

    #[getter]
    fn board_is_empty(&self) -> bool {
        self.inner.board.is_empty()
    }

    #[getter]
    fn turn_order(&self) -> Vec<PlayerColor> {
        self.inner
            .turn_order
            .colors()
            .into_iter()
            .map(PlayerColor::from_core)
            .collect()
    }

    #[getter]
    fn current_color(&self) -> PlayerColor {
        PlayerColor::from_core(self.inner.turn.current_color())
    }

    #[getter]
    fn player_slots(&self) -> PlayerSlots {
        PlayerSlots::from_core(self.inner.player_slots)
    }

    fn __repr__(&self) -> String {
        format!(
            "GameState(game_id={:?}, mode={:?}, status={:?}, version={})",
            self.game_id(),
            self.mode(),
            self.status(),
            self.version()
        )
    }
}
