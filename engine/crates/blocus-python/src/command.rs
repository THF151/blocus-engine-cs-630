use crate::errors::input_error;
use crate::types::PlayerColor;
use blocus_core::{
    BoardIndex, CommandId, GameId, InputError as CoreInputError, OrientationId, PassCommand,
    PieceId, PlaceCommand, PlayerId,
};
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use uuid::Uuid;

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPlaceCommand>()?;
    module.add_class::<PyPassCommand>()?;

    Ok(())
}

#[pyclass(name = "PlaceCommand", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PyPlaceCommand {
    inner: PlaceCommand,
}

impl PyPlaceCommand {
    pub const fn as_core(&self) -> PlaceCommand {
        self.inner
    }
}

#[pymethods]
#[allow(clippy::too_many_arguments)]
impl PyPlaceCommand {
    #[new]
    #[pyo3(signature = (
        command_id,
        game_id,
        player_id,
        color,
        piece_id,
        orientation_id,
        row,
        col
    ))]
    fn new(
        py: Python<'_>,
        command_id: &str,
        game_id: &str,
        player_id: &str,
        color: PlayerColor,
        piece_id: u8,
        orientation_id: u8,
        row: u8,
        col: u8,
    ) -> PyResult<Self> {
        let command_id = parse_command_id(py, command_id)?;
        let game_id = parse_game_id(py, game_id)?;
        let player_id = parse_player_id(py, player_id)?;
        let piece_id = parse_piece_id(py, piece_id)?;
        let orientation_id = parse_orientation_id(py, orientation_id)?;
        let anchor = parse_board_index(py, row, col)?;

        Ok(Self {
            inner: PlaceCommand {
                command_id,
                game_id,
                player_id,
                color: color.as_core(),
                piece_id,
                orientation_id,
                anchor,
            },
        })
    }

    #[getter]
    fn command_id(&self) -> String {
        self.inner.command_id.to_string()
    }

    #[getter]
    fn game_id(&self) -> String {
        self.inner.game_id.to_string()
    }

    #[getter]
    fn player_id(&self) -> String {
        self.inner.player_id.to_string()
    }

    #[getter]
    fn color(&self) -> PlayerColor {
        PlayerColor::from_core(self.inner.color)
    }

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

    fn __repr__(&self) -> String {
        format!(
            "PlaceCommand(command_id='{}', game_id='{}', player_id='{}', color={}, piece_id={}, orientation_id={}, row={}, col={})",
            self.command_id(),
            self.game_id(),
            self.player_id(),
            self.color().repr(),
            self.piece_id(),
            self.orientation_id(),
            self.row(),
            self.col()
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

#[pyclass(name = "PassCommand", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PyPassCommand {
    inner: PassCommand,
}

impl PyPassCommand {
    pub const fn as_core(&self) -> PassCommand {
        self.inner
    }
}

#[pymethods]
impl PyPassCommand {
    #[new]
    #[pyo3(signature = (command_id, game_id, player_id, color))]
    fn new(
        py: Python<'_>,
        command_id: &str,
        game_id: &str,
        player_id: &str,
        color: PlayerColor,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: PassCommand {
                command_id: parse_command_id(py, command_id)?,
                game_id: parse_game_id(py, game_id)?,
                player_id: parse_player_id(py, player_id)?,
                color: color.as_core(),
            },
        })
    }

    #[getter]
    fn command_id(&self) -> String {
        self.inner.command_id.to_string()
    }

    #[getter]
    fn game_id(&self) -> String {
        self.inner.game_id.to_string()
    }

    #[getter]
    fn player_id(&self) -> String {
        self.inner.player_id.to_string()
    }

    #[getter]
    fn color(&self) -> PlayerColor {
        PlayerColor::from_core(self.inner.color)
    }

    fn __repr__(&self) -> String {
        format!(
            "PassCommand(command_id='{}', game_id='{}', player_id='{}', color={})",
            self.command_id(),
            self.game_id(),
            self.player_id(),
            self.color().repr()
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

fn parse_uuid(py: Python<'_>, value: &str) -> PyResult<Uuid> {
    match Uuid::parse_str(value) {
        Ok(uuid) => Ok(uuid),
        Err(_) => Err(input_error(py, CoreInputError::InvalidGameConfig)),
    }
}

fn parse_game_id(py: Python<'_>, value: &str) -> PyResult<GameId> {
    parse_uuid(py, value).map(GameId::from_uuid)
}

fn parse_player_id(py: Python<'_>, value: &str) -> PyResult<PlayerId> {
    parse_uuid(py, value).map(PlayerId::from_uuid)
}

fn parse_command_id(py: Python<'_>, value: &str) -> PyResult<CommandId> {
    parse_uuid(py, value).map(CommandId::from_uuid)
}

fn parse_piece_id(py: Python<'_>, value: u8) -> PyResult<PieceId> {
    match PieceId::try_new(value) {
        Ok(piece_id) => Ok(piece_id),
        Err(_) => Err(input_error(py, CoreInputError::UnknownPiece)),
    }
}

fn parse_orientation_id(py: Python<'_>, value: u8) -> PyResult<OrientationId> {
    match OrientationId::try_new(value) {
        Ok(orientation_id) => Ok(orientation_id),
        Err(_) => Err(input_error(py, CoreInputError::UnknownOrientation)),
    }
}

fn parse_board_index(py: Python<'_>, row: u8, col: u8) -> PyResult<BoardIndex> {
    match BoardIndex::from_row_col(row, col) {
        Ok(index) => Ok(index),
        Err(error) => Err(input_error(py, error)),
    }
}
