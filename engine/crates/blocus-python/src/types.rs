//! Python wrappers for stable public enum contract types.

use crate::errors::input_error;
use blocus_core::{
    GameStatus as CoreGameStatus, InputError as CoreInputError, PlayerColor as CorePlayerColor,
    ScoringMode as CoreScoringMode,
};
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyModule;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PlayerColor>()?;
    module.add_class::<GameStatus>()?;
    module.add_class::<ScoringMode>()?;

    Ok(())
}

#[pyclass(name = "PlayerColor", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PlayerColor {
    inner: CorePlayerColor,
}

impl PlayerColor {
    pub const fn from_core(inner: CorePlayerColor) -> Self {
        Self { inner }
    }

    pub const fn as_core(self) -> CorePlayerColor {
        self.inner
    }

    pub const fn repr(self) -> &'static str {
        match self.inner {
            CorePlayerColor::Blue => "PlayerColor.BLUE",
            CorePlayerColor::Yellow => "PlayerColor.YELLOW",
            CorePlayerColor::Red => "PlayerColor.RED",
            CorePlayerColor::Green => "PlayerColor.GREEN",
        }
    }
}

#[pymethods]
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "PyO3 pyclass methods must borrow Python-owned objects"
)]
impl PlayerColor {
    #[new]
    fn new(py: Python<'_>, value: &str) -> PyResult<Self> {
        match value {
            "blue" => Ok(Self::from_core(CorePlayerColor::Blue)),
            "yellow" => Ok(Self::from_core(CorePlayerColor::Yellow)),
            "red" => Ok(Self::from_core(CorePlayerColor::Red)),
            "green" => Ok(Self::from_core(CorePlayerColor::Green)),
            _ => Err(input_error(py, CoreInputError::InvalidGameConfig)),
        }
    }

    #[classattr]
    #[pyo3(name = "BLUE")]
    fn blue() -> Self {
        Self::from_core(CorePlayerColor::Blue)
    }

    #[classattr]
    #[pyo3(name = "YELLOW")]
    fn yellow() -> Self {
        Self::from_core(CorePlayerColor::Yellow)
    }

    #[classattr]
    #[pyo3(name = "RED")]
    fn red() -> Self {
        Self::from_core(CorePlayerColor::Red)
    }

    #[classattr]
    #[pyo3(name = "GREEN")]
    fn green() -> Self {
        Self::from_core(CorePlayerColor::Green)
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self.inner {
            CorePlayerColor::Blue => "BLUE",
            CorePlayerColor::Yellow => "YELLOW",
            CorePlayerColor::Red => "RED",
            CorePlayerColor::Green => "GREEN",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.inner.as_str()
    }

    #[getter]
    fn index(&self) -> usize {
        self.inner.index()
    }

    fn __str__(&self) -> &'static str {
        self.inner.as_str()
    }

    fn __repr__(&self) -> &'static str {
        self.repr()
    }

    fn __hash__(&self) -> isize {
        match self.inner {
            CorePlayerColor::Blue => 0,
            CorePlayerColor::Yellow => 1,
            CorePlayerColor::Red => 2,
            CorePlayerColor::Green => 3,
        }
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}

#[pyclass(name = "GameStatus", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct GameStatus {
    inner: CoreGameStatus,
}

impl GameStatus {
    pub const fn from_core(inner: CoreGameStatus) -> Self {
        Self { inner }
    }

    pub const fn as_core(self) -> CoreGameStatus {
        self.inner
    }

    const fn name_value_hash(self) -> (&'static str, &'static str, isize) {
        match self.inner {
            CoreGameStatus::InProgress => ("IN_PROGRESS", "in_progress", 0),
            CoreGameStatus::Finished => ("FINISHED", "finished", 1),
            _ => ("UNKNOWN", "unknown", -1),
        }
    }
}

#[pymethods]
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "PyO3 pyclass methods must borrow Python-owned objects"
)]
impl GameStatus {
    #[new]
    fn new(py: Python<'_>, value: &str) -> PyResult<Self> {
        match value {
            "in_progress" => Ok(Self::from_core(CoreGameStatus::InProgress)),
            "finished" => Ok(Self::from_core(CoreGameStatus::Finished)),
            _ => Err(input_error(py, CoreInputError::InvalidGameConfig)),
        }
    }

    #[classattr]
    #[pyo3(name = "IN_PROGRESS")]
    fn in_progress() -> Self {
        Self::from_core(CoreGameStatus::InProgress)
    }

    #[classattr]
    #[pyo3(name = "FINISHED")]
    fn finished() -> Self {
        Self::from_core(CoreGameStatus::Finished)
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.name_value_hash().0
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.name_value_hash().1
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }

    fn __repr__(&self) -> String {
        format!("GameStatus.{}", self.name())
    }

    fn __hash__(&self) -> isize {
        self.name_value_hash().2
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}

#[pyclass(name = "ScoringMode", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ScoringMode {
    inner: CoreScoringMode,
}

impl ScoringMode {
    pub const fn from_core(inner: CoreScoringMode) -> Self {
        Self { inner }
    }

    pub const fn as_core(self) -> CoreScoringMode {
        self.inner
    }

    const fn name_value_hash(self) -> (&'static str, &'static str, isize) {
        match self.inner {
            CoreScoringMode::Basic => ("BASIC", "basic", 0),
            CoreScoringMode::Advanced => ("ADVANCED", "advanced", 1),
            _ => ("UNKNOWN", "unknown", -1),
        }
    }
}

#[pymethods]
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "PyO3 pyclass methods must borrow Python-owned objects"
)]
impl ScoringMode {
    #[new]
    fn new(py: Python<'_>, value: &str) -> PyResult<Self> {
        match value {
            "basic" => Ok(Self::from_core(CoreScoringMode::Basic)),
            "advanced" => Ok(Self::from_core(CoreScoringMode::Advanced)),
            _ => Err(input_error(py, CoreInputError::InvalidGameConfig)),
        }
    }

    #[classattr]
    #[pyo3(name = "BASIC")]
    fn basic() -> Self {
        Self::from_core(CoreScoringMode::Basic)
    }

    #[classattr]
    #[pyo3(name = "ADVANCED")]
    fn advanced() -> Self {
        Self::from_core(CoreScoringMode::Advanced)
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.name_value_hash().0
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.name_value_hash().1
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }

    fn __repr__(&self) -> String {
        format!("ScoringMode.{}", self.name())
    }

    fn __hash__(&self) -> isize {
        self.name_value_hash().2
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}
