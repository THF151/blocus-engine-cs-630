use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;

#[pyclass(name = "PlayerColor", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PlayerColor {
    inner: blocus_core::PlayerColor,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl PlayerColor {
    pub const fn from_core(inner: blocus_core::PlayerColor) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> blocus_core::PlayerColor {
        self.inner
    }

    fn parse(value: &str) -> PyResult<blocus_core::PlayerColor> {
        match value {
            "blue" | "BLUE" | "Blue" => Ok(blocus_core::PlayerColor::Blue),
            "yellow" | "YELLOW" | "Yellow" => Ok(blocus_core::PlayerColor::Yellow),
            "red" | "RED" | "Red" => Ok(blocus_core::PlayerColor::Red),
            "green" | "GREEN" | "Green" => Ok(blocus_core::PlayerColor::Green),
            "black" | "BLACK" | "Black" => Ok(blocus_core::PlayerColor::Black),
            "white" | "WHITE" | "White" => Ok(blocus_core::PlayerColor::White),
            _ => {
                let _ = value;
                Err(crate::conversion::invalid_game_config_error())
            }
        }
    }

    pub fn repr(&self) -> &'static str {
        match self.inner {
            blocus_core::PlayerColor::Blue => "PlayerColor.BLUE",
            blocus_core::PlayerColor::Yellow => "PlayerColor.YELLOW",
            blocus_core::PlayerColor::Red => "PlayerColor.RED",
            blocus_core::PlayerColor::Green => "PlayerColor.GREEN",
            blocus_core::PlayerColor::Black => "PlayerColor.BLACK",
            blocus_core::PlayerColor::White => "PlayerColor.WHITE",
        }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl PlayerColor {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Ok(Self::from_core(Self::parse(value)?))
    }

    #[staticmethod]
    fn blue() -> Self {
        Self::from_core(blocus_core::PlayerColor::Blue)
    }

    #[staticmethod]
    fn yellow() -> Self {
        Self::from_core(blocus_core::PlayerColor::Yellow)
    }

    #[staticmethod]
    fn red() -> Self {
        Self::from_core(blocus_core::PlayerColor::Red)
    }

    #[staticmethod]
    fn green() -> Self {
        Self::from_core(blocus_core::PlayerColor::Green)
    }

    #[staticmethod]
    fn black() -> Self {
        Self::from_core(blocus_core::PlayerColor::Black)
    }

    #[staticmethod]
    fn white() -> Self {
        Self::from_core(blocus_core::PlayerColor::White)
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self.inner {
            blocus_core::PlayerColor::Blue => "BLUE",
            blocus_core::PlayerColor::Yellow => "YELLOW",
            blocus_core::PlayerColor::Red => "RED",
            blocus_core::PlayerColor::Green => "GREEN",
            blocus_core::PlayerColor::Black => "BLACK",
            blocus_core::PlayerColor::White => "WHITE",
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
            blocus_core::PlayerColor::Blue => 0,
            blocus_core::PlayerColor::Yellow => 1,
            blocus_core::PlayerColor::Red => 2,
            blocus_core::PlayerColor::Green => 3,
            blocus_core::PlayerColor::Black => 4,
            blocus_core::PlayerColor::White => 5,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn __richcmp__(&self, other: PyRef<'_, Self>, op: CompareOp) -> bool {
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
    inner: blocus_core::GameStatus,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl GameStatus {
    pub const fn from_core(inner: blocus_core::GameStatus) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> blocus_core::GameStatus {
        self.inner
    }

    fn parse(value: &str) -> PyResult<blocus_core::GameStatus> {
        match value {
            "in_progress" | "IN_PROGRESS" | "InProgress" => Ok(blocus_core::GameStatus::InProgress),
            "finished" | "FINISHED" | "Finished" => Ok(blocus_core::GameStatus::Finished),
            _ => {
                let _ = value;
                Err(crate::conversion::invalid_game_config_error())
            }
        }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl GameStatus {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Ok(Self::from_core(Self::parse(value)?))
    }

    #[staticmethod]
    fn in_progress() -> Self {
        Self::from_core(blocus_core::GameStatus::InProgress)
    }

    #[staticmethod]
    fn finished() -> Self {
        Self::from_core(blocus_core::GameStatus::Finished)
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self.inner {
            blocus_core::GameStatus::InProgress => "IN_PROGRESS",
            blocus_core::GameStatus::Finished => "FINISHED",
            _ => "UNKNOWN",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self.inner {
            blocus_core::GameStatus::InProgress => "in_progress",
            blocus_core::GameStatus::Finished => "finished",
            _ => "unknown",
        }
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }

    fn __repr__(&self) -> String {
        format!("GameStatus.{}", self.name())
    }

    fn __hash__(&self) -> isize {
        match self.inner {
            blocus_core::GameStatus::InProgress => 0,
            blocus_core::GameStatus::Finished => 1,
            _ => -1,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn __richcmp__(&self, other: PyRef<'_, Self>, op: CompareOp) -> bool {
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
    inner: blocus_core::ScoringMode,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl ScoringMode {
    pub const fn from_core(inner: blocus_core::ScoringMode) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> blocus_core::ScoringMode {
        self.inner
    }

    fn parse(value: &str) -> PyResult<blocus_core::ScoringMode> {
        match value {
            "basic" | "BASIC" | "Basic" => Ok(blocus_core::ScoringMode::Basic),
            "advanced" | "ADVANCED" | "Advanced" => Ok(blocus_core::ScoringMode::Advanced),
            _ => {
                let _ = value;
                Err(crate::conversion::invalid_game_config_error())
            }
        }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl ScoringMode {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        Ok(Self::from_core(Self::parse(value)?))
    }

    #[staticmethod]
    fn basic() -> Self {
        Self::from_core(blocus_core::ScoringMode::Basic)
    }

    #[staticmethod]
    fn advanced() -> Self {
        Self::from_core(blocus_core::ScoringMode::Advanced)
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self.inner {
            blocus_core::ScoringMode::Basic => "BASIC",
            blocus_core::ScoringMode::Advanced => "ADVANCED",
            _ => "UNKNOWN",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self.inner {
            blocus_core::ScoringMode::Basic => "basic",
            blocus_core::ScoringMode::Advanced => "advanced",
            _ => "unknown",
        }
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }

    fn __repr__(&self) -> String {
        format!("ScoringMode.{}", self.name())
    }

    fn __hash__(&self) -> isize {
        match self.inner {
            blocus_core::ScoringMode::Basic => 0,
            blocus_core::ScoringMode::Advanced => 1,
            _ => -1,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn __richcmp__(&self, other: PyRef<'_, Self>, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.inner == other.inner,
            CompareOp::Ne => self.inner != other.inner,
            _ => false,
        }
    }
}
