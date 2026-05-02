use crate::state::GameState;
use crate::types::ScoringMode;
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<LegalMove>()?;
    module.add_class::<DomainEventKind>()?;
    module.add_class::<DomainResponseKind>()?;
    module.add_class::<DomainEvent>()?;
    module.add_class::<DomainResponse>()?;
    module.add_class::<GameResult>()?;
    module.add_class::<ScoreEntry>()?;
    module.add_class::<ScoreBoard>()?;

    let py = module.py();

    let domain_event_kind = module.getattr("DomainEventKind")?;
    domain_event_kind.setattr(
        "MOVE_APPLIED",
        Py::new(
            py,
            DomainEventKind::from_core(blocus_core::DomainEventKind::MoveApplied),
        )?,
    )?;
    domain_event_kind.setattr(
        "PLAYER_PASSED",
        Py::new(
            py,
            DomainEventKind::from_core(blocus_core::DomainEventKind::PlayerPassed),
        )?,
    )?;
    domain_event_kind.setattr(
        "TURN_ADVANCED",
        Py::new(
            py,
            DomainEventKind::from_core(blocus_core::DomainEventKind::TurnAdvanced),
        )?,
    )?;
    domain_event_kind.setattr(
        "GAME_FINISHED",
        Py::new(
            py,
            DomainEventKind::from_core(blocus_core::DomainEventKind::GameFinished),
        )?,
    )?;

    let domain_response_kind = module.getattr("DomainResponseKind")?;
    domain_response_kind.setattr(
        "MOVE_APPLIED",
        Py::new(
            py,
            DomainResponseKind::from_core(blocus_core::DomainResponseKind::MoveApplied),
        )?,
    )?;
    domain_response_kind.setattr(
        "PLAYER_PASSED",
        Py::new(
            py,
            DomainResponseKind::from_core(blocus_core::DomainResponseKind::PlayerPassed),
        )?,
    )?;
    domain_response_kind.setattr(
        "GAME_FINISHED",
        Py::new(
            py,
            DomainResponseKind::from_core(blocus_core::DomainResponseKind::GameFinished),
        )?,
    )?;

    Ok(())
}

#[pyclass(name = "LegalMove", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LegalMove {
    inner: blocus_core::LegalMove,
}

impl LegalMove {
    pub const fn from_core(inner: blocus_core::LegalMove) -> Self {
        Self { inner }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl LegalMove {
    #[getter]
    fn piece_id(&self) -> u8 {
        self.inner.piece_id.as_u8()
    }

    #[getter]
    fn orientation_id(&self) -> u8 {
        self.inner.orientation_id.as_u8()
    }

    #[getter]
    fn row(&self) -> u8 {
        self.inner.anchor.row()
    }

    #[getter]
    fn col(&self) -> u8 {
        self.inner.anchor.col()
    }

    #[getter]
    fn board_index(&self) -> u16 {
        self.inner.anchor.bit_index()
    }

    #[getter]
    fn score_delta(&self) -> u8 {
        self.inner.score_delta
    }

    fn __repr__(&self) -> String {
        format!(
            "LegalMove(piece_id={}, orientation_id={}, row={}, col={}, score_delta={})",
            self.piece_id(),
            self.orientation_id(),
            self.row(),
            self.col(),
            self.score_delta()
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}

#[pyclass(name = "DomainEventKind", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DomainEventKind {
    inner: blocus_core::DomainEventKind,
}

impl DomainEventKind {
    pub const fn from_core(inner: blocus_core::DomainEventKind) -> Self {
        Self { inner }
    }

    pub const fn as_core(self) -> blocus_core::DomainEventKind {
        self.inner
    }

    fn parse(value: &str) -> PyResult<blocus_core::DomainEventKind> {
        match value {
            "move_applied" | "MOVE_APPLIED" | "MoveApplied" => {
                Ok(blocus_core::DomainEventKind::MoveApplied)
            }
            "player_passed" | "PLAYER_PASSED" | "PlayerPassed" => {
                Ok(blocus_core::DomainEventKind::PlayerPassed)
            }
            "turn_advanced" | "TURN_ADVANCED" | "TurnAdvanced" => {
                Ok(blocus_core::DomainEventKind::TurnAdvanced)
            }
            "game_finished" | "GAME_FINISHED" | "GameFinished" => {
                Ok(blocus_core::DomainEventKind::GameFinished)
            }
            _ => Err(crate::conversion::invalid_game_config_error()),
        }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl DomainEventKind {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Ok(Self::from_core(Self::parse(value)?))
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self.inner {
            blocus_core::DomainEventKind::MoveApplied => "MOVE_APPLIED",
            blocus_core::DomainEventKind::PlayerPassed => "PLAYER_PASSED",
            blocus_core::DomainEventKind::TurnAdvanced => "TURN_ADVANCED",
            blocus_core::DomainEventKind::GameFinished => "GAME_FINISHED",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self.inner {
            blocus_core::DomainEventKind::MoveApplied => "move_applied",
            blocus_core::DomainEventKind::PlayerPassed => "player_passed",
            blocus_core::DomainEventKind::TurnAdvanced => "turn_advanced",
            blocus_core::DomainEventKind::GameFinished => "game_finished",
        }
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }

    fn __repr__(&self) -> String {
        format!("DomainEventKind.{}", self.name())
    }

    fn __hash__(&self) -> isize {
        match self.inner {
            blocus_core::DomainEventKind::MoveApplied => 0,
            blocus_core::DomainEventKind::PlayerPassed => 1,
            blocus_core::DomainEventKind::TurnAdvanced => 2,
            blocus_core::DomainEventKind::GameFinished => 3,
        }
    }

    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> bool {
        if let Ok(other_kind) = other.extract::<PyRef<'_, Self>>() {
            return match op {
                CompareOp::Eq => self.inner == other_kind.inner,
                CompareOp::Ne => self.inner != other_kind.inner,
                _ => false,
            };
        }

        if let Ok(other_value) = other.extract::<String>() {
            return match op {
                CompareOp::Eq => self.value() == other_value,
                CompareOp::Ne => self.value() != other_value,
                _ => false,
            };
        }

        matches!(op, CompareOp::Ne)
    }
}

#[pyclass(name = "DomainResponseKind", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DomainResponseKind {
    inner: blocus_core::DomainResponseKind,
}

impl DomainResponseKind {
    pub const fn from_core(inner: blocus_core::DomainResponseKind) -> Self {
        Self { inner }
    }

    pub const fn as_core(self) -> blocus_core::DomainResponseKind {
        self.inner
    }

    fn parse(value: &str) -> PyResult<blocus_core::DomainResponseKind> {
        match value {
            "move_applied" | "MOVE_APPLIED" | "MoveApplied" => {
                Ok(blocus_core::DomainResponseKind::MoveApplied)
            }
            "player_passed" | "PLAYER_PASSED" | "PlayerPassed" => {
                Ok(blocus_core::DomainResponseKind::PlayerPassed)
            }
            "game_finished" | "GAME_FINISHED" | "GameFinished" => {
                Ok(blocus_core::DomainResponseKind::GameFinished)
            }
            _ => Err(crate::conversion::invalid_game_config_error()),
        }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl DomainResponseKind {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Ok(Self::from_core(Self::parse(value)?))
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self.inner {
            blocus_core::DomainResponseKind::MoveApplied => "MOVE_APPLIED",
            blocus_core::DomainResponseKind::PlayerPassed => "PLAYER_PASSED",
            blocus_core::DomainResponseKind::GameFinished => "GAME_FINISHED",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self.inner {
            blocus_core::DomainResponseKind::MoveApplied => "move_applied",
            blocus_core::DomainResponseKind::PlayerPassed => "player_passed",
            blocus_core::DomainResponseKind::GameFinished => "game_finished",
        }
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }

    fn __repr__(&self) -> String {
        format!("DomainResponseKind.{}", self.name())
    }

    fn __hash__(&self) -> isize {
        match self.inner {
            blocus_core::DomainResponseKind::MoveApplied => 0,
            blocus_core::DomainResponseKind::PlayerPassed => 1,
            blocus_core::DomainResponseKind::GameFinished => 2,
        }
    }

    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> bool {
        if let Ok(other_kind) = other.extract::<PyRef<'_, Self>>() {
            return match op {
                CompareOp::Eq => self.inner == other_kind.inner,
                CompareOp::Ne => self.inner != other_kind.inner,
                _ => false,
            };
        }

        if let Ok(other_value) = other.extract::<String>() {
            return match op {
                CompareOp::Eq => self.value() == other_value,
                CompareOp::Ne => self.value() != other_value,
                _ => false,
            };
        }

        matches!(op, CompareOp::Ne)
    }
}

#[pyclass(name = "DomainEvent", frozen, skip_from_py_object)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DomainEvent {
    inner: blocus_core::DomainEvent,
}

impl DomainEvent {
    pub const fn from_core(inner: blocus_core::DomainEvent) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl DomainEvent {
    #[getter]
    fn kind(&self) -> DomainEventKind {
        DomainEventKind::from_core(self.inner.kind)
    }

    #[getter]
    fn game_id(&self) -> String {
        self.inner.game_id.to_string()
    }

    #[getter]
    fn version(&self) -> u64 {
        self.inner.version.as_u64()
    }

    fn __repr__(&self) -> String {
        format!(
            "DomainEvent(kind={}, game_id='{}', version={})",
            self.kind().value(),
            self.game_id(),
            self.version()
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}

#[pyclass(name = "DomainResponse", frozen, skip_from_py_object)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DomainResponse {
    inner: blocus_core::DomainResponse,
}

impl DomainResponse {
    pub const fn from_core(inner: blocus_core::DomainResponse) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl DomainResponse {
    #[getter]
    fn kind(&self) -> DomainResponseKind {
        DomainResponseKind::from_core(self.inner.kind)
    }

    #[getter]
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "DomainResponse(kind={}, message='{}')",
            self.kind().value(),
            self.message()
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}

#[pyclass(name = "GameResult", frozen, skip_from_py_object)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GameResult {
    inner: blocus_core::GameResult,
}

impl GameResult {
    pub const fn from_core(inner: blocus_core::GameResult) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl GameResult {
    #[getter]
    fn next_state(&self) -> GameState {
        GameState::from_core(self.inner.next_state.clone())
    }

    #[getter]
    fn events(&self) -> Vec<DomainEvent> {
        self.inner
            .events
            .iter()
            .cloned()
            .map(DomainEvent::from_core)
            .collect()
    }

    #[getter]
    fn response(&self) -> DomainResponse {
        DomainResponse::from_core(self.inner.response.clone())
    }

    fn __repr__(&self) -> String {
        format!(
            "GameResult(next_state={:?}, events={}, response={:?})",
            self.inner.next_state,
            self.inner.events.len(),
            self.inner.response
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}

#[pyclass(name = "ScoreEntry", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ScoreEntry {
    inner: blocus_core::ScoreEntry,
}

impl ScoreEntry {
    pub const fn from_core(inner: blocus_core::ScoreEntry) -> Self {
        Self { inner }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl ScoreEntry {
    #[getter]
    fn player_id(&self) -> String {
        self.inner.player_id.to_string()
    }

    #[getter]
    fn score(&self) -> i16 {
        self.inner.score
    }

    fn __repr__(&self) -> String {
        format!(
            "ScoreEntry(player_id='{}', score={})",
            self.player_id(),
            self.score()
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}

#[pyclass(name = "ScoreBoard", frozen, skip_from_py_object)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ScoreBoard {
    inner: blocus_core::ScoreBoard,
}

impl ScoreBoard {
    pub const fn from_core(inner: blocus_core::ScoreBoard) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl ScoreBoard {
    #[getter]
    fn scoring(&self) -> ScoringMode {
        ScoringMode::from_core(self.inner.scoring)
    }

    #[getter]
    fn entries(&self) -> Vec<ScoreEntry> {
        self.inner
            .entries
            .iter()
            .copied()
            .map(ScoreEntry::from_core)
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "ScoreBoard(scoring={:?}, entries={})",
            self.scoring(),
            self.inner.entries.len()
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}
