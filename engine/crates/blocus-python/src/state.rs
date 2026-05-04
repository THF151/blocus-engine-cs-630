use crate::config::{GameMode, PlayerSlots};
use crate::conversion::{map_input_error, parse_uuid};
use crate::types::{GameStatus, PlayerColor, ScoringMode};
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use serde_json::{Value, json};

#[pyclass(name = "BoardCell", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BoardCell {
    row: u8,
    col: u8,
    board_index: u16,
    color: PlayerColor,
}

impl BoardCell {
    fn from_index(color: PlayerColor, index: blocus_core::BoardIndex) -> Self {
        Self {
            row: index.row(),
            col: index.col(),
            board_index: index.bit_index(),
            color,
        }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl BoardCell {
    #[getter]
    fn row(&self) -> u8 {
        self.row
    }

    #[getter]
    fn col(&self) -> u8 {
        self.col
    }

    #[getter]
    fn board_index(&self) -> u16 {
        self.board_index
    }

    #[getter]
    fn color(&self) -> PlayerColor {
        self.color
    }

    fn __repr__(&self) -> String {
        format!(
            "BoardCell(row={}, col={}, board_index={}, color={})",
            self.row,
            self.col,
            self.board_index,
            self.color.repr()
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self == other,
            CompareOp::Ne => self != other,
            _ => false,
        }
    }
}

#[pyclass(name = "BoardSnapshot", frozen, skip_from_py_object)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoardSnapshot {
    blue: Vec<BoardCell>,
    yellow: Vec<BoardCell>,
    red: Vec<BoardCell>,
    green: Vec<BoardCell>,
    black: Vec<BoardCell>,
    white: Vec<BoardCell>,
}

impl BoardSnapshot {
    fn from_board(board: &blocus_core::BoardState) -> Self {
        Self {
            blue: cells_for_color(board, blocus_core::PlayerColor::Blue),
            yellow: cells_for_color(board, blocus_core::PlayerColor::Yellow),
            red: cells_for_color(board, blocus_core::PlayerColor::Red),
            green: cells_for_color(board, blocus_core::PlayerColor::Green),
            black: cells_for_color(board, blocus_core::PlayerColor::Black),
            white: cells_for_color(board, blocus_core::PlayerColor::White),
        }
    }
}

#[pymethods]
impl BoardSnapshot {
    #[getter]
    fn blue(&self) -> Vec<BoardCell> {
        self.blue.clone()
    }

    #[getter]
    fn yellow(&self) -> Vec<BoardCell> {
        self.yellow.clone()
    }

    #[getter]
    fn red(&self) -> Vec<BoardCell> {
        self.red.clone()
    }

    #[getter]
    fn green(&self) -> Vec<BoardCell> {
        self.green.clone()
    }

    #[getter]
    fn black(&self) -> Vec<BoardCell> {
        self.black.clone()
    }

    #[getter]
    fn white(&self) -> Vec<BoardCell> {
        self.white.clone()
    }

    #[getter]
    fn occupied(&self) -> Vec<BoardCell> {
        self.blue
            .iter()
            .chain(self.yellow.iter())
            .chain(self.red.iter())
            .chain(self.green.iter())
            .chain(self.black.iter())
            .chain(self.white.iter())
            .copied()
            .collect()
    }

    #[getter]
    fn occupied_count(&self) -> usize {
        self.blue.len()
            + self.yellow.len()
            + self.red.len()
            + self.green.len()
            + self.black.len()
            + self.white.len()
    }

    fn cells_for_color(&self, color: PlayerColor) -> Vec<BoardCell> {
        match color.as_core() {
            blocus_core::PlayerColor::Blue => self.blue(),
            blocus_core::PlayerColor::Yellow => self.yellow(),
            blocus_core::PlayerColor::Red => self.red(),
            blocus_core::PlayerColor::Green => self.green(),
            blocus_core::PlayerColor::Black => self.black(),
            blocus_core::PlayerColor::White => self.white(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "BoardSnapshot(blue={}, yellow={}, red={}, green={}, black={}, white={})",
            self.blue.len(),
            self.yellow.len(),
            self.red.len(),
            self.green.len(),
            self.black.len(),
            self.white.len()
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self == other,
            CompareOp::Ne => self != other,
            _ => false,
        }
    }
}

#[pyclass(name = "Piece", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Piece {
    id: u8,
    name: &'static str,
    square_count: u8,
    orientation_count: u8,
}

impl Piece {
    fn from_core(piece: blocus_core::CanonicalPiece) -> Self {
        Self {
            id: piece.id().as_u8(),
            name: piece.name(),
            square_count: piece.square_count(),
            orientation_count: piece.orientation_count(),
        }
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl Piece {
    #[getter]
    fn id(&self) -> u8 {
        self.id
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.name
    }

    #[getter]
    fn square_count(&self) -> u8 {
        self.square_count
    }

    #[getter]
    fn orientation_count(&self) -> u8 {
        self.orientation_count
    }

    fn __repr__(&self) -> String {
        format!(
            "Piece(id={}, name='{}', square_count={}, orientation_count={})",
            self.id, self.name, self.square_count, self.orientation_count
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self == other,
            CompareOp::Ne => self != other,
            _ => false,
        }
    }
}

#[pyclass(name = "InventorySummary", frozen, skip_from_py_object)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InventorySummary {
    color: PlayerColor,
    used_piece_ids: Vec<u16>,
    available_piece_ids: Vec<u16>,
    used_square_count: u16,
    remaining_square_count: u16,
}

impl InventorySummary {
    fn from_state(state: &blocus_core::GameState, color: blocus_core::PlayerColor) -> Self {
        let used_piece_ids = used_piece_ids_for(state, color);
        let available_piece_ids = available_piece_ids_for(state, color);

        Self {
            color: PlayerColor::from_core(color),
            used_square_count: square_count_for_piece_ids(&used_piece_ids),
            remaining_square_count: square_count_for_piece_ids(&available_piece_ids),
            used_piece_ids,
            available_piece_ids,
        }
    }
}

#[pymethods]
impl InventorySummary {
    #[getter]
    fn color(&self) -> PlayerColor {
        self.color
    }

    #[getter]
    fn used_piece_ids(&self) -> Vec<u16> {
        self.used_piece_ids.clone()
    }

    #[getter]
    fn available_piece_ids(&self) -> Vec<u16> {
        self.available_piece_ids.clone()
    }

    #[getter]
    fn used_pieces(&self) -> Vec<Piece> {
        piece_ids_to_pieces(&self.used_piece_ids)
    }

    #[getter]
    fn available_pieces(&self) -> Vec<Piece> {
        piece_ids_to_pieces(&self.available_piece_ids)
    }

    #[getter]
    fn used_count(&self) -> usize {
        self.used_piece_ids.len()
    }

    #[getter]
    fn available_count(&self) -> usize {
        self.available_piece_ids.len()
    }

    #[getter]
    fn used_square_count(&self) -> u16 {
        self.used_square_count
    }

    #[getter]
    fn remaining_square_count(&self) -> u16 {
        self.remaining_square_count
    }

    #[getter]
    fn is_complete(&self) -> bool {
        self.available_piece_ids.is_empty()
    }

    fn __repr__(&self) -> String {
        format!(
            "InventorySummary(color={}, used_count={}, available_count={}, remaining_square_count={})",
            self.color.repr(),
            self.used_piece_ids.len(),
            self.available_piece_ids.len(),
            self.remaining_square_count
        )
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self == other,
            CompareOp::Ne => self != other,
            _ => false,
        }
    }
}

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
    #[staticmethod]
    fn from_json(json_text: &str) -> PyResult<Self> {
        let value: Value = serde_json::from_str(json_text).map_err(|error| {
            pyo3::exceptions::PyValueError::new_err(format!("invalid GameState JSON: {error}"))
        })?;

        let mut state = parse_game_state_json(&value)?;
        blocus_core::validate_game_state(&state).map_err(map_state_json_validation_error)?;
        state.hash = blocus_core::compute_hash_full(&state);

        Ok(Self::from_core(state))
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&state_to_json(&self.inner)).map_err(|error| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "failed to serialize GameState: {error}"
            ))
        })
    }

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
    fn board_size(&self) -> u8 {
        self.inner.mode.board_size()
    }

    #[getter]
    fn board(&self) -> BoardSnapshot {
        BoardSnapshot::from_board(&self.inner.board)
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

    fn cell(&self, row: u8, col: u8) -> PyResult<Option<PlayerColor>> {
        if row >= self.inner.mode.board_size() || col >= self.inner.mode.board_size() {
            return Err(map_input_error(blocus_core::InputError::InvalidBoardIndex));
        }

        let index = blocus_core::BoardIndex::from_row_col(row, col).map_err(map_input_error)?;
        Ok(cell_color_at(&self.inner.board, index))
    }

    #[pyo3(signature = (color=None))]
    fn occupied_cells(&self, color: Option<PlayerColor>) -> Vec<BoardCell> {
        match color {
            Some(color) => cells_for_color(&self.inner.board, color.as_core()),
            None => BoardSnapshot::from_board(&self.inner.board).occupied(),
        }
    }

    fn board_matrix(&self) -> Vec<Vec<Option<PlayerColor>>> {
        (0..self.inner.mode.board_size())
            .map(|row| {
                (0..self.inner.mode.board_size())
                    .map(|col| {
                        let index = blocus_core::BoardIndex::from_row_col(row, col)
                            .unwrap_or_else(|_| unreachable!("row and column are playable"));
                        cell_color_at(&self.inner.board, index)
                    })
                    .collect()
            })
            .collect()
    }

    fn board_counts(&self) -> Vec<(PlayerColor, u32)> {
        self.inner
            .mode
            .active_colors()
            .iter()
            .copied()
            .map(|color| {
                (
                    PlayerColor::from_core(color),
                    self.inner.board.occupied(color).count(),
                )
            })
            .collect()
    }

    fn cells_for_color(&self, color: PlayerColor) -> Vec<BoardCell> {
        cells_for_color(&self.inner.board, color.as_core())
    }

    fn used_piece_ids(&self, color: PlayerColor) -> Vec<u16> {
        used_piece_ids_for(&self.inner, color.as_core())
    }

    fn available_piece_ids(&self, color: PlayerColor) -> Vec<u16> {
        available_piece_ids_for(&self.inner, color.as_core())
    }

    fn used_pieces(&self, color: PlayerColor) -> Vec<Piece> {
        piece_ids_to_pieces(&used_piece_ids_for(&self.inner, color.as_core()))
    }

    fn available_pieces(&self, color: PlayerColor) -> Vec<Piece> {
        piece_ids_to_pieces(&available_piece_ids_for(&self.inner, color.as_core()))
    }

    fn inventory_summary(&self, color: PlayerColor) -> InventorySummary {
        InventorySummary::from_state(&self.inner, color.as_core())
    }

    fn remaining_square_count(&self, color: PlayerColor) -> u16 {
        square_count_for_piece_ids(&available_piece_ids_for(&self.inner, color.as_core()))
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

fn state_to_json(state: &blocus_core::GameState) -> Value {
    json!({
        "schema_version": state.schema_version.as_u16(),
        "game_id": state.game_id.to_string(),
        "mode": game_mode_value(state.mode),
        "scoring": scoring_mode_to_str(state.scoring),
        "turn_order": state
            .turn_order
            .colors()
            .into_iter()
            .map(player_color_value)
            .collect::<Vec<_>>(),
        "player_slots": player_slots_to_json(state.player_slots),
        "board": board_to_json(state.mode, &state.board),
        "inventories": inventories_to_json(state),
        "last_piece_by_color_packed": state.last_piece_by_color.packed(),
        "turn": {
            "current_color": player_color_value(state.turn.current_color()),
            "passed_mask": state.turn.passed_mask(),
            "shared_color_turn_index": state.turn.shared_color_turn_index(),
        },
        "status": game_status_to_str(state.status),
        "version": state.version.as_u64(),
        "hash": state.hash.as_u64(),
    })
}

fn player_slots_to_json(slots: blocus_core::PlayerSlots) -> Value {
    json!({
        "controllers": slots
            .mode()
            .active_colors()
            .iter()
            .copied()
            .map(|color| {
                slots
                    .controller_of(color)
                    .map(|player| player.to_string())
            })
            .collect::<Vec<_>>(),
        "shared_color_turn": slots.shared_color_turn().map(|shared| {
            json!({
                "color": player_color_value(shared.color()),
                "players": shared
                    .players()
                    .into_iter()
                    .map(|player| player.to_string())
                    .collect::<Vec<_>>(),
            })
        }),
    })
}

fn board_to_json(mode: blocus_core::GameMode, board: &blocus_core::BoardState) -> Value {
    let occupied_by_color = board.occupied_by_color();

    match mode {
        blocus_core::GameMode::TwoPlayer
        | blocus_core::GameMode::ThreePlayer
        | blocus_core::GameMode::FourPlayer => json!({
            "blue": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Blue.index()]),
            "yellow": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Yellow.index()]),
            "red": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Red.index()]),
            "green": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Green.index()]),
        }),
        blocus_core::GameMode::Duo => json!({
            "black": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Black.index()]),
            "white": mask_to_json(occupied_by_color[blocus_core::PlayerColor::White.index()]),
        }),
    }
}

fn mask_to_json(mask: blocus_core::BoardMask) -> Value {
    let lanes = mask
        .lanes()
        .into_iter()
        .map(|lane| lane.to_string())
        .collect::<Vec<_>>();

    json!({
        "lanes": lanes,
        "count": mask.count(),
    })
}

fn inventories_to_json(state: &blocus_core::GameState) -> Value {
    json!(
        state
            .mode
            .active_colors()
            .iter()
            .copied()
            .map(|color| state.inventories[color.index()].used_mask())
            .collect::<Vec<_>>()
    )
}

fn parse_game_state_json(value: &Value) -> PyResult<blocus_core::GameState> {
    let schema_version =
        blocus_core::StateSchemaVersion::new(required_u16(value, "schema_version")?);
    let game_id =
        blocus_core::GameId::from_uuid(parse_uuid(required_str(value, "game_id")?, "game_id")?);
    let mode = parse_game_mode(required_str(value, "mode")?)?;
    let scoring = parse_scoring_mode(required_str(value, "scoring")?)?;
    let turn_order = parse_turn_order(mode, required_array(value, "turn_order")?)?;
    let player_slots = parse_player_slots(mode, required_object_value(value, "player_slots")?)?;
    let board = parse_board(mode, required_object_value(value, "board")?)?;
    let inventories = parse_inventories(mode, required_array(value, "inventories")?)?;
    let last_piece_by_color = blocus_core::LastPieceByColor::from_packed(required_u32(
        value,
        "last_piece_by_color_packed",
    )?);

    let turn_value = required_object_value(value, "turn")?;
    let current_color = parse_player_color(required_str(turn_value, "current_color")?)?;
    let passed_mask = required_u8(turn_value, "passed_mask")?;
    let shared_color_turn_index = required_usize(turn_value, "shared_color_turn_index")?;
    let turn =
        blocus_core::TurnState::from_parts(current_color, passed_mask, shared_color_turn_index);

    let status = parse_game_status(required_str(value, "status")?)?;
    let version = blocus_core::StateVersion::new(required_u64(value, "version")?);

    Ok(blocus_core::GameState {
        schema_version,
        game_id,
        mode,
        scoring,
        turn_order,
        player_slots,
        board,
        inventories,
        last_piece_by_color,
        turn,
        status,
        version,
        hash: blocus_core::ZobristHash::ZERO,
    })
}

fn parse_turn_order(
    mode: blocus_core::GameMode,
    values: &[Value],
) -> PyResult<blocus_core::TurnOrder> {
    match mode {
        blocus_core::GameMode::TwoPlayer
        | blocus_core::GameMode::ThreePlayer
        | blocus_core::GameMode::FourPlayer => {
            let colors = parse_exact_colors::<4>(values, "turn_order")?;
            blocus_core::TurnOrder::try_new(colors).map_err(map_input_error)
        }
        blocus_core::GameMode::Duo => {
            let colors = parse_exact_colors::<2>(values, "turn_order")?;
            let order = blocus_core::TurnOrder::duo(colors[0]).map_err(map_input_error)?;

            if order.colors() == colors {
                Ok(order)
            } else {
                Err(crate::conversion::invalid_game_config_error())
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn parse_player_slots(
    mode: blocus_core::GameMode,
    value: &Value,
) -> PyResult<blocus_core::PlayerSlots> {
    let controllers = required_array(value, "controllers")?;

    if controllers.len() != mode.active_colors().len() {
        return Err(crate::conversion::invalid_game_config_error());
    }

    let controller_at_active_index = |index: usize| -> PyResult<Option<blocus_core::PlayerId>> {
        if controllers[index].is_null() {
            Ok(None)
        } else {
            Ok(Some(blocus_core::PlayerId::from_uuid(parse_uuid(
                controllers[index]
                    .as_str()
                    .ok_or_else(crate::conversion::invalid_game_config_error)?,
                "controller",
            )?)))
        }
    };

    let controller = |color: blocus_core::PlayerColor| -> PyResult<Option<blocus_core::PlayerId>> {
        let Some(active_index) = mode
            .active_colors()
            .iter()
            .position(|active| *active == color)
        else {
            return Ok(None);
        };

        controller_at_active_index(active_index)
    };

    match mode {
        blocus_core::GameMode::TwoPlayer => {
            let Some(blue_red_player) = controller(blocus_core::PlayerColor::Blue)? else {
                return Err(crate::conversion::invalid_game_config_error());
            };
            let Some(yellow_green_player) = controller(blocus_core::PlayerColor::Yellow)? else {
                return Err(crate::conversion::invalid_game_config_error());
            };

            let Some(red_player) = controller(blocus_core::PlayerColor::Red)? else {
                return Err(crate::conversion::invalid_game_config_error());
            };
            let Some(green_player) = controller(blocus_core::PlayerColor::Green)? else {
                return Err(crate::conversion::invalid_game_config_error());
            };

            if blue_red_player != red_player || yellow_green_player != green_player {
                return Err(crate::conversion::invalid_game_config_error());
            }

            blocus_core::PlayerSlots::two_player(blue_red_player, yellow_green_player)
                .map_err(map_input_error)
        }
        blocus_core::GameMode::ThreePlayer => {
            let shared_value = value
                .get("shared_color_turn")
                .filter(|value| !value.is_null())
                .ok_or_else(crate::conversion::invalid_game_config_error)?;

            let shared_color = parse_player_color(required_str(shared_value, "color")?)?;
            let shared_players = required_array(shared_value, "players")?;

            if shared_players.len() != 3 {
                return Err(crate::conversion::invalid_game_config_error());
            }

            let shared = blocus_core::SharedColorTurn::try_new(
                shared_color,
                [
                    blocus_core::PlayerId::from_uuid(parse_uuid(
                        shared_players[0]
                            .as_str()
                            .ok_or_else(crate::conversion::invalid_game_config_error)?,
                        "shared_players[0]",
                    )?),
                    blocus_core::PlayerId::from_uuid(parse_uuid(
                        shared_players[1]
                            .as_str()
                            .ok_or_else(crate::conversion::invalid_game_config_error)?,
                        "shared_players[1]",
                    )?),
                    blocus_core::PlayerId::from_uuid(parse_uuid(
                        shared_players[2]
                            .as_str()
                            .ok_or_else(crate::conversion::invalid_game_config_error)?,
                        "shared_players[2]",
                    )?),
                ],
            )
            .map_err(map_input_error)?;

            let mut owned = Vec::with_capacity(3);

            for color in blocus_core::PlayerColor::CLASSIC {
                if color == shared_color {
                    if controller(color)?.is_some() {
                        return Err(crate::conversion::invalid_game_config_error());
                    }

                    continue;
                }

                let Some(player_id) = controller(color)? else {
                    return Err(crate::conversion::invalid_game_config_error());
                };

                owned.push((color, player_id));
            }

            let owned: [(blocus_core::PlayerColor, blocus_core::PlayerId); 3] = owned
                .try_into()
                .map_err(|_values| crate::conversion::invalid_game_config_error())?;

            blocus_core::PlayerSlots::three_player(owned, shared).map_err(map_input_error)
        }
        blocus_core::GameMode::FourPlayer => {
            let assignments = [
                required_assignment(&controller, blocus_core::PlayerColor::Blue)?,
                required_assignment(&controller, blocus_core::PlayerColor::Yellow)?,
                required_assignment(&controller, blocus_core::PlayerColor::Red)?,
                required_assignment(&controller, blocus_core::PlayerColor::Green)?,
            ];

            blocus_core::PlayerSlots::four_player(assignments).map_err(map_input_error)
        }
        blocus_core::GameMode::Duo => {
            let Some(black_player) = controller(blocus_core::PlayerColor::Black)? else {
                return Err(crate::conversion::invalid_game_config_error());
            };
            let Some(white_player) = controller(blocus_core::PlayerColor::White)? else {
                return Err(crate::conversion::invalid_game_config_error());
            };

            blocus_core::PlayerSlots::duo(black_player, white_player).map_err(map_input_error)
        }
    }
}

fn required_assignment<F>(
    controller: &F,
    color: blocus_core::PlayerColor,
) -> PyResult<(blocus_core::PlayerColor, blocus_core::PlayerId)>
where
    F: Fn(blocus_core::PlayerColor) -> PyResult<Option<blocus_core::PlayerId>>,
{
    let Some(player_id) = controller(color)? else {
        return Err(crate::conversion::invalid_game_config_error());
    };

    Ok((color, player_id))
}

fn parse_board(mode: blocus_core::GameMode, value: &Value) -> PyResult<blocus_core::BoardState> {
    let board = value
        .as_object()
        .ok_or_else(crate::conversion::invalid_game_config_error)?;

    let mut masks = [blocus_core::BoardMask::EMPTY; blocus_core::MAX_PLAYER_COLOR_COUNT];

    for color in mode.active_colors().iter().copied() {
        masks[color.index()] = parse_mask(
            board
                .get(color.as_str())
                .ok_or_else(crate::conversion::invalid_game_config_error)?,
        )?;
    }

    Ok(blocus_core::BoardState::from_occupied_by_color(masks))
}

fn parse_mask(value: &Value) -> PyResult<blocus_core::BoardMask> {
    let lanes = value
        .get("lanes")
        .and_then(Value::as_array)
        .ok_or_else(crate::conversion::invalid_game_config_error)?;

    if lanes.len() != blocus_core::BOARD_LANES {
        return Err(crate::conversion::invalid_game_config_error());
    }

    let mut parsed_lanes = [0u128; blocus_core::BOARD_LANES];

    for (lane_index, lane_value) in lanes.iter().enumerate() {
        let lane_text = lane_value
            .as_str()
            .ok_or_else(crate::conversion::invalid_game_config_error)?;

        parsed_lanes[lane_index] = lane_text
            .parse::<u128>()
            .map_err(|_| crate::conversion::invalid_game_config_error())?;
    }

    blocus_core::BoardMask::try_from_lanes(parsed_lanes).map_err(|_| {
        crate::conversion::map_domain_error(blocus_core::EngineError::CorruptedState.into())
    })
}

fn parse_inventories(
    mode: blocus_core::GameMode,
    values: &[Value],
) -> PyResult<[blocus_core::PieceInventory; blocus_core::MAX_PLAYER_COLOR_COUNT]> {
    if values.len() != mode.active_colors().len() {
        return Err(crate::conversion::invalid_game_config_error());
    }

    let mut inventories = [blocus_core::PieceInventory::EMPTY; blocus_core::MAX_PLAYER_COLOR_COUNT];

    for (index, color) in mode.active_colors().iter().copied().enumerate() {
        let raw = value_as_u32(&values[index])?;

        if raw & !blocus_core::ALL_PIECES_MASK != 0 {
            return Err(crate::conversion::invalid_game_config_error());
        }

        inventories[color.index()] = blocus_core::PieceInventory::from_used_mask(raw);
    }

    Ok(inventories)
}

fn parse_exact_colors<const N: usize>(
    values: &[Value],
    _field_name: &str,
) -> PyResult<[blocus_core::PlayerColor; N]> {
    if values.len() != N {
        return Err(crate::conversion::invalid_game_config_error());
    }

    let mut colors = [blocus_core::PlayerColor::Blue; N];

    for (index, value) in values.iter().enumerate() {
        colors[index] = parse_player_color(
            value
                .as_str()
                .ok_or_else(crate::conversion::invalid_game_config_error)?,
        )?;
    }

    Ok(colors)
}

fn required_object_value<'a>(value: &'a Value, field: &str) -> PyResult<&'a Value> {
    value
        .get(field)
        .filter(|value| value.is_object())
        .ok_or_else(crate::conversion::invalid_game_config_error)
}

fn required_array<'a>(value: &'a Value, field: &str) -> PyResult<&'a [Value]> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(crate::conversion::invalid_game_config_error)
}

fn required_str<'a>(value: &'a Value, field: &str) -> PyResult<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(crate::conversion::invalid_game_config_error)
}

fn required_u8(value: &Value, field: &str) -> PyResult<u8> {
    u8::try_from(required_u64(value, field)?)
        .map_err(|_| crate::conversion::invalid_game_config_error())
}

fn required_u16(value: &Value, field: &str) -> PyResult<u16> {
    u16::try_from(required_u64(value, field)?)
        .map_err(|_| crate::conversion::invalid_game_config_error())
}

fn required_u32(value: &Value, field: &str) -> PyResult<u32> {
    u32::try_from(required_u64(value, field)?)
        .map_err(|_| crate::conversion::invalid_game_config_error())
}

fn required_usize(value: &Value, field: &str) -> PyResult<usize> {
    usize::try_from(required_u64(value, field)?)
        .map_err(|_| crate::conversion::invalid_game_config_error())
}

fn required_u64(value: &Value, field: &str) -> PyResult<u64> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(crate::conversion::invalid_game_config_error)
}

fn value_as_u32(value: &Value) -> PyResult<u32> {
    let raw = value
        .as_u64()
        .ok_or_else(crate::conversion::invalid_game_config_error)?;

    u32::try_from(raw).map_err(|_| crate::conversion::invalid_game_config_error())
}

fn parse_player_color(value: &str) -> PyResult<blocus_core::PlayerColor> {
    match value {
        "blue" | "BLUE" | "Blue" => Ok(blocus_core::PlayerColor::Blue),
        "yellow" | "YELLOW" | "Yellow" => Ok(blocus_core::PlayerColor::Yellow),
        "red" | "RED" | "Red" => Ok(blocus_core::PlayerColor::Red),
        "green" | "GREEN" | "Green" => Ok(blocus_core::PlayerColor::Green),
        "black" | "BLACK" | "Black" => Ok(blocus_core::PlayerColor::Black),
        "white" | "WHITE" | "White" => Ok(blocus_core::PlayerColor::White),
        _ => Err(crate::conversion::invalid_game_config_error()),
    }
}

fn parse_game_mode(value: &str) -> PyResult<blocus_core::GameMode> {
    match value {
        "two_player" | "TWO_PLAYER" | "TwoPlayer" => Ok(blocus_core::GameMode::TwoPlayer),
        "three_player" | "THREE_PLAYER" | "ThreePlayer" => Ok(blocus_core::GameMode::ThreePlayer),
        "four_player" | "FOUR_PLAYER" | "FourPlayer" => Ok(blocus_core::GameMode::FourPlayer),
        "duo" | "DUO" | "Duo" => Ok(blocus_core::GameMode::Duo),
        _ => Err(crate::conversion::invalid_game_config_error()),
    }
}

fn parse_scoring_mode(value: &str) -> PyResult<blocus_core::ScoringMode> {
    match value {
        "basic" | "BASIC" | "Basic" => Ok(blocus_core::ScoringMode::Basic),
        "advanced" | "ADVANCED" | "Advanced" => Ok(blocus_core::ScoringMode::Advanced),
        _ => Err(crate::conversion::invalid_game_config_error()),
    }
}

fn parse_game_status(value: &str) -> PyResult<blocus_core::GameStatus> {
    match value {
        "in_progress" | "IN_PROGRESS" | "InProgress" => Ok(blocus_core::GameStatus::InProgress),
        "finished" | "FINISHED" | "Finished" => Ok(blocus_core::GameStatus::Finished),
        _ => Err(crate::conversion::invalid_game_config_error()),
    }
}

fn player_color_value(color: blocus_core::PlayerColor) -> &'static str {
    color.as_str()
}

fn game_mode_value(mode: blocus_core::GameMode) -> &'static str {
    match mode {
        blocus_core::GameMode::TwoPlayer => "two_player",
        blocus_core::GameMode::ThreePlayer => "three_player",
        blocus_core::GameMode::FourPlayer => "four_player",
        blocus_core::GameMode::Duo => "duo",
    }
}

fn scoring_mode_to_str(scoring: blocus_core::ScoringMode) -> &'static str {
    match scoring {
        blocus_core::ScoringMode::Basic => "basic",
        blocus_core::ScoringMode::Advanced => "advanced",
        _ => "unknown",
    }
}

fn game_status_to_str(status: blocus_core::GameStatus) -> &'static str {
    match status {
        blocus_core::GameStatus::InProgress => "in_progress",
        blocus_core::GameStatus::Finished => "finished",
        _ => "unknown",
    }
}

fn cells_for_color(
    board: &blocus_core::BoardState,
    color: blocus_core::PlayerColor,
) -> Vec<BoardCell> {
    let py_color = PlayerColor::from_core(color);
    let mask = board.occupied(color);
    let mut cells = Vec::with_capacity(mask.count() as usize);

    for row in 0..blocus_core::BOARD_SIZE {
        for col in 0..blocus_core::BOARD_SIZE {
            let index = blocus_core::BoardIndex::from_row_col(row, col)
                .unwrap_or_else(|_| unreachable!("row and column are within playable board"));

            if mask.contains(index) {
                cells.push(BoardCell::from_index(py_color, index));
            }
        }
    }

    cells
}

fn cell_color_at(
    board: &blocus_core::BoardState,
    index: blocus_core::BoardIndex,
) -> Option<PlayerColor> {
    blocus_core::PlayerColor::ALL
        .into_iter()
        .find(|color| board.occupied(*color).contains(index))
        .map(PlayerColor::from_core)
}

fn used_piece_ids_for(state: &blocus_core::GameState, color: blocus_core::PlayerColor) -> Vec<u16> {
    let inventory = state.inventories[color.index()];

    (0..blocus_core::PIECE_COUNT)
        .filter(|piece_id| {
            let piece = blocus_core::PieceId::try_new(*piece_id)
                .unwrap_or_else(|_| unreachable!("piece id in official range is valid"));

            inventory.is_used(piece)
        })
        .map(u16::from)
        .collect()
}

fn available_piece_ids_for(
    state: &blocus_core::GameState,
    color: blocus_core::PlayerColor,
) -> Vec<u16> {
    let inventory = state.inventories[color.index()];

    (0..blocus_core::PIECE_COUNT)
        .filter(|piece_id| {
            let piece = blocus_core::PieceId::try_new(*piece_id)
                .unwrap_or_else(|_| unreachable!("piece id in official range is valid"));

            inventory.is_available(piece)
        })
        .map(u16::from)
        .collect()
}

fn piece_ids_to_pieces(piece_ids: &[u16]) -> Vec<Piece> {
    piece_ids
        .iter()
        .copied()
        .map(|raw_piece_id| {
            let raw_piece_id = u8::try_from(raw_piece_id)
                .unwrap_or_else(|_| unreachable!("piece id came from official range"));
            let piece_id = blocus_core::PieceId::try_new(raw_piece_id)
                .unwrap_or_else(|_| unreachable!("piece id came from official range"));
            Piece::from_core(*blocus_core::standard_piece(piece_id))
        })
        .collect()
}

fn square_count_for_piece_ids(piece_ids: &[u16]) -> u16 {
    piece_ids
        .iter()
        .copied()
        .map(|raw_piece_id| {
            let raw_piece_id = u8::try_from(raw_piece_id)
                .unwrap_or_else(|_| unreachable!("piece id came from official range"));
            let piece_id = blocus_core::PieceId::try_new(raw_piece_id)
                .unwrap_or_else(|_| unreachable!("piece id came from official range"));
            u16::from(blocus_core::standard_piece(piece_id).square_count())
        })
        .sum()
}

#[allow(clippy::needless_pass_by_value)]
fn map_state_json_validation_error(error: blocus_core::DomainError) -> PyErr {
    crate::conversion::map_domain_error(error)
}
