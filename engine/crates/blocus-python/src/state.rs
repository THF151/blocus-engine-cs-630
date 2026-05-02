use crate::config::{GameMode, PlayerSlots};
use crate::conversion::{map_input_error, parse_uuid};
use crate::types::{GameStatus, PlayerColor, ScoringMode};
use pyo3::prelude::*;
use serde_json::{Value, json};

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
        "board": board_to_json(&state.board),
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
        "controllers": blocus_core::PlayerColor::ALL
            .into_iter()
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

fn board_to_json(board: &blocus_core::BoardState) -> Value {
    let occupied_by_color = board.occupied_by_color();

    json!({
        "blue": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Blue.index()]),
        "yellow": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Yellow.index()]),
        "red": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Red.index()]),
        "green": mask_to_json(occupied_by_color[blocus_core::PlayerColor::Green.index()]),
    })
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
            .inventories
            .into_iter()
            .map(blocus_core::PieceInventory::used_mask)
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
    let turn_order = parse_turn_order(required_array(value, "turn_order")?)?;
    let player_slots = parse_player_slots(mode, required_object_value(value, "player_slots")?)?;
    let board = parse_board(required_object_value(value, "board")?)?;
    let inventories = parse_inventories(required_array(value, "inventories")?)?;
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

    let status = parse_game_status(required_str(value, "status")?);
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

fn parse_turn_order(values: &[Value]) -> PyResult<blocus_core::TurnOrder> {
    let colors = parse_exact_colors::<4>(values, "turn_order")?;
    blocus_core::TurnOrder::try_new(colors).map_err(map_input_error)
}

fn parse_player_slots(
    mode: blocus_core::GameMode,
    value: &Value,
) -> PyResult<blocus_core::PlayerSlots> {
    let controllers = required_array(value, "controllers")?;

    if controllers.len() != blocus_core::PLAYER_COLOR_COUNT {
        return Err(crate::conversion::invalid_game_config_error());
    }

    let controller = |index: usize| -> PyResult<Option<blocus_core::PlayerId>> {
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

    match mode {
        blocus_core::GameMode::TwoPlayer => {
            let Some(blue_red_player) = controller(blocus_core::PlayerColor::Blue.index())? else {
                return Err(crate::conversion::invalid_game_config_error());
            };
            let Some(yellow_green_player) = controller(blocus_core::PlayerColor::Yellow.index())?
            else {
                return Err(crate::conversion::invalid_game_config_error());
            };

            let Some(red_player) = controller(blocus_core::PlayerColor::Red.index())? else {
                return Err(crate::conversion::invalid_game_config_error());
            };
            let Some(green_player) = controller(blocus_core::PlayerColor::Green.index())? else {
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

            for color in blocus_core::PlayerColor::ALL {
                if color == shared_color {
                    if controller(color.index())?.is_some() {
                        return Err(crate::conversion::invalid_game_config_error());
                    }

                    continue;
                }

                let Some(player_id) = controller(color.index())? else {
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
    }
}

fn required_assignment<F>(
    controller: &F,
    color: blocus_core::PlayerColor,
) -> PyResult<(blocus_core::PlayerColor, blocus_core::PlayerId)>
where
    F: Fn(usize) -> PyResult<Option<blocus_core::PlayerId>>,
{
    let Some(player_id) = controller(color.index())? else {
        return Err(crate::conversion::invalid_game_config_error());
    };

    Ok((color, player_id))
}

fn parse_board(value: &Value) -> PyResult<blocus_core::BoardState> {
    let board = value
        .as_object()
        .ok_or_else(crate::conversion::invalid_game_config_error)?;

    let masks = [
        parse_mask(
            board
                .get("blue")
                .ok_or_else(crate::conversion::invalid_game_config_error)?,
        )?,
        parse_mask(
            board
                .get("yellow")
                .ok_or_else(crate::conversion::invalid_game_config_error)?,
        )?,
        parse_mask(
            board
                .get("red")
                .ok_or_else(crate::conversion::invalid_game_config_error)?,
        )?,
        parse_mask(
            board
                .get("green")
                .ok_or_else(crate::conversion::invalid_game_config_error)?,
        )?,
    ];

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

    Ok(blocus_core::BoardMask::from_lanes(parsed_lanes))
}

fn parse_inventories(
    values: &[Value],
) -> PyResult<[blocus_core::PieceInventory; blocus_core::PLAYER_COLOR_COUNT]> {
    if values.len() != blocus_core::PLAYER_COLOR_COUNT {
        return Err(crate::conversion::invalid_game_config_error());
    }

    Ok([
        blocus_core::PieceInventory::from_used_mask(value_as_u32(&values[0])?),
        blocus_core::PieceInventory::from_used_mask(value_as_u32(&values[1])?),
        blocus_core::PieceInventory::from_used_mask(value_as_u32(&values[2])?),
        blocus_core::PieceInventory::from_used_mask(value_as_u32(&values[3])?),
    ])
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
        _ => Err(crate::conversion::invalid_game_config_error()),
    }
}

fn parse_game_mode(value: &str) -> PyResult<blocus_core::GameMode> {
    match value {
        "two_player" | "TWO_PLAYER" | "TwoPlayer" => Ok(blocus_core::GameMode::TwoPlayer),
        "three_player" | "THREE_PLAYER" | "ThreePlayer" => Ok(blocus_core::GameMode::ThreePlayer),
        "four_player" | "FOUR_PLAYER" | "FourPlayer" => Ok(blocus_core::GameMode::FourPlayer),
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

fn parse_game_status(value: &str) -> blocus_core::GameStatus {
    match value {
        "finished" | "FINISHED" | "Finished" => blocus_core::GameStatus::Finished,
        _ => blocus_core::GameStatus::InProgress,
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
