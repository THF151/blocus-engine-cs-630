use crate::state::GameState;
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyModule;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<DomainEvent>()?;
    module.add_class::<DomainResponse>()?;
    module.add_class::<GameResult>()?;

    Ok(())
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
    fn kind(&self) -> &'static str {
        event_kind_name(self.inner.kind)
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
            "DomainEvent(kind='{}', game_id='{}', version={})",
            self.kind(),
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
    fn kind(&self) -> &'static str {
        response_kind_name(self.inner.kind)
    }

    #[getter]
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "DomainResponse(kind='{}', message='{}')",
            self.kind(),
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

const fn event_kind_name(kind: blocus_core::DomainEventKind) -> &'static str {
    match kind {
        blocus_core::DomainEventKind::MoveApplied => "move_applied",
        blocus_core::DomainEventKind::PlayerPassed => "player_passed",
        blocus_core::DomainEventKind::TurnAdvanced => "turn_advanced",
        blocus_core::DomainEventKind::GameFinished => "game_finished",
        _ => "unknown",
    }
}

const fn response_kind_name(kind: blocus_core::DomainResponseKind) -> &'static str {
    match kind {
        blocus_core::DomainResponseKind::MoveApplied => "move_applied",
        blocus_core::DomainResponseKind::PlayerPassed => "player_passed",
        blocus_core::DomainResponseKind::GameFinished => "game_finished",
        _ => "unknown",
    }
}
