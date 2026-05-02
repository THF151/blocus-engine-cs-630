#![allow(clippy::trivially_copy_pass_by_ref)]

use crate::errors::input_error;
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyModule;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Piece>()?;
    module.add_class::<PieceOrientation>()?;

    Ok(())
}

#[pyclass(name = "Piece", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Piece {
    inner: blocus_core::CanonicalPiece,
}

impl Piece {
    pub const fn from_core(inner: blocus_core::CanonicalPiece) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl Piece {
    #[getter]
    fn id(&self) -> u8 {
        self.inner.id().as_u8()
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    #[getter]
    fn square_count(&self) -> u8 {
        self.inner.square_count()
    }

    #[getter]
    fn orientation_count(&self) -> u8 {
        self.inner.orientation_count()
    }

    #[getter]
    fn orientations(&self) -> Vec<PieceOrientation> {
        self.inner
            .orientations()
            .into_iter()
            .map(PieceOrientation::from_core)
            .collect()
    }

    fn orientation(&self, py: Python<'_>, orientation_id: u8) -> PyResult<PieceOrientation> {
        let orientation_id = parse_orientation_id(py, orientation_id)?;

        self.inner
            .orientation(orientation_id)
            .map(PieceOrientation::from_core)
            .ok_or_else(|| input_error(py, blocus_core::InputError::UnknownOrientation))
    }

    fn __repr__(&self) -> String {
        format!(
            "Piece(id={}, name='{}', square_count={}, orientation_count={})",
            self.id(),
            self.name(),
            self.square_count(),
            self.orientation_count()
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

#[pyclass(name = "PieceOrientation", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PieceOrientation {
    inner: blocus_core::PieceOrientation,
}

impl PieceOrientation {
    pub const fn from_core(inner: blocus_core::PieceOrientation) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PieceOrientation {
    #[getter]
    fn id(&self) -> u8 {
        self.inner.id().as_u8()
    }

    #[getter]
    fn width(&self) -> u8 {
        self.inner.shape().width()
    }

    #[getter]
    fn height(&self) -> u8 {
        self.inner.shape().height()
    }

    #[getter]
    fn square_count(&self) -> u8 {
        self.inner.shape().square_count()
    }

    #[getter]
    fn cells(&self) -> Vec<(u8, u8)> {
        self.inner.shape().cells()
    }

    fn cells_at(&self, py: Python<'_>, row: u8, col: u8) -> PyResult<Vec<(u8, u8)>> {
        if row
            .checked_add(self.height())
            .is_none_or(|end| end > blocus_core::BOARD_SIZE)
            || col
                .checked_add(self.width())
                .is_none_or(|end| end > blocus_core::BOARD_SIZE)
        {
            return Err(input_error(py, blocus_core::InputError::InvalidBoardIndex));
        }

        Ok(self
            .cells()
            .into_iter()
            .map(|(local_row, local_col)| (row + local_row, col + local_col))
            .collect())
    }

    fn __repr__(&self) -> String {
        format!(
            "PieceOrientation(id={}, width={}, height={}, square_count={})",
            self.id(),
            self.width(),
            self.height(),
            self.square_count()
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

fn parse_orientation_id(py: Python<'_>, value: u8) -> PyResult<blocus_core::OrientationId> {
    blocus_core::OrientationId::try_new(value)
        .map_err(|_| input_error(py, blocus_core::InputError::UnknownOrientation))
}
