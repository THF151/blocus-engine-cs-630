#![allow(clippy::trivially_copy_pass_by_ref)]
use crate::conversion::{map_input_error, parse_player_id, parse_three_players, parse_uuid};
use crate::types::{PlayerColor, ScoringMode};
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;

#[pyclass(name = "GameMode", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct GameMode {
    inner: blocus_core::GameMode,
}

impl GameMode {
    pub const fn from_core(inner: blocus_core::GameMode) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> blocus_core::GameMode {
        self.inner
    }
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl GameMode {
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        match value {
            "two_player" | "TWO_PLAYER" | "TwoPlayer" => {
                Ok(Self::from_core(blocus_core::GameMode::TwoPlayer))
            }
            "three_player" | "THREE_PLAYER" | "ThreePlayer" => {
                Ok(Self::from_core(blocus_core::GameMode::ThreePlayer))
            }
            "four_player" | "FOUR_PLAYER" | "FourPlayer" => {
                Ok(Self::from_core(blocus_core::GameMode::FourPlayer))
            }
            _ => {
                let _ = value;
                Err(crate::conversion::invalid_game_config_error())
            }
        }
    }

    #[staticmethod]
    fn two_player() -> Self {
        Self::from_core(blocus_core::GameMode::TwoPlayer)
    }

    #[staticmethod]
    fn three_player() -> Self {
        Self::from_core(blocus_core::GameMode::ThreePlayer)
    }

    #[staticmethod]
    fn four_player() -> Self {
        Self::from_core(blocus_core::GameMode::FourPlayer)
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self.inner {
            blocus_core::GameMode::TwoPlayer => "TWO_PLAYER",
            blocus_core::GameMode::ThreePlayer => "THREE_PLAYER",
            blocus_core::GameMode::FourPlayer => "FOUR_PLAYER",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self.inner {
            blocus_core::GameMode::TwoPlayer => "two_player",
            blocus_core::GameMode::ThreePlayer => "three_player",
            blocus_core::GameMode::FourPlayer => "four_player",
        }
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }

    fn __repr__(&self) -> String {
        format!("GameMode.{}", self.name())
    }

    fn __hash__(&self) -> isize {
        match self.inner {
            blocus_core::GameMode::TwoPlayer => 2,
            blocus_core::GameMode::ThreePlayer => 3,
            blocus_core::GameMode::FourPlayer => 4,
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

#[pyclass(name = "SharedColorTurn", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SharedColorTurn {
    inner: blocus_core::SharedColorTurn,
}

impl SharedColorTurn {
    pub const fn from_core(inner: blocus_core::SharedColorTurn) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> blocus_core::SharedColorTurn {
        self.inner
    }
}

#[pymethods]
impl SharedColorTurn {
    #[new]
    fn new(color: &PlayerColor, players: Vec<String>) -> PyResult<Self> {
        let players = parse_three_players(players)?;
        blocus_core::SharedColorTurn::try_new(color.as_core(), players)
            .map(Self::from_core)
            .map_err(map_input_error)
    }

    #[getter]
    fn color(&self) -> PlayerColor {
        PlayerColor::from_core(self.inner.color())
    }

    #[getter]
    fn players(&self) -> Vec<String> {
        self.inner
            .players()
            .into_iter()
            .map(|player| player.to_string())
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "SharedColorTurn(color={:?}, players={:?})",
            self.color(),
            self.players()
        )
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

#[pyclass(name = "PlayerSlots", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PlayerSlots {
    inner: blocus_core::PlayerSlots,
}

impl PlayerSlots {
    pub const fn from_core(inner: blocus_core::PlayerSlots) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> blocus_core::PlayerSlots {
        self.inner
    }
}

#[pymethods]
impl PlayerSlots {
    #[staticmethod]
    fn two_player(blue_red_player: &str, yellow_green_player: &str) -> PyResult<Self> {
        let blue_red_player = parse_player_id(blue_red_player, "blue_red_player")?;
        let yellow_green_player = parse_player_id(yellow_green_player, "yellow_green_player")?;

        blocus_core::PlayerSlots::two_player(blue_red_player, yellow_green_player)
            .map(Self::from_core)
            .map_err(map_input_error)
    }

    #[staticmethod]
    fn three_player(
        owned_colors: Vec<(PlayerColor, String)>,
        shared_color_turn: &SharedColorTurn,
    ) -> PyResult<Self> {
        let [
            (first_color, first_player),
            (second_color, second_player),
            (third_color, third_player),
        ]: [(PlayerColor, String); 3] =
            owned_colors
                .try_into()
                .map_err(|values: Vec<(PlayerColor, String)>| {
                    pyo3::exceptions::PyValueError::new_err(format!(
                        "owned_colors must contain exactly 3 entries; got {}",
                        values.len()
                    ))
                })?;

        let assignments = [
            (
                first_color.as_core(),
                parse_player_id(&first_player, "owned_colors[0].player")?,
            ),
            (
                second_color.as_core(),
                parse_player_id(&second_player, "owned_colors[1].player")?,
            ),
            (
                third_color.as_core(),
                parse_player_id(&third_player, "owned_colors[2].player")?,
            ),
        ];

        blocus_core::PlayerSlots::three_player(assignments, shared_color_turn.as_core())
            .map(Self::from_core)
            .map_err(map_input_error)
    }

    #[staticmethod]
    fn four_player(assignments: Vec<(PlayerColor, String)>) -> PyResult<Self> {
        let [
            (first_color, first_player),
            (second_color, second_player),
            (third_color, third_player),
            (fourth_color, fourth_player),
        ]: [(PlayerColor, String); 4] =
            assignments
                .try_into()
                .map_err(|values: Vec<(PlayerColor, String)>| {
                    pyo3::exceptions::PyValueError::new_err(format!(
                        "assignments must contain exactly 4 entries; got {}",
                        values.len()
                    ))
                })?;

        let assignments = [
            (
                first_color.as_core(),
                parse_player_id(&first_player, "assignments[0].player")?,
            ),
            (
                second_color.as_core(),
                parse_player_id(&second_player, "assignments[1].player")?,
            ),
            (
                third_color.as_core(),
                parse_player_id(&third_player, "assignments[2].player")?,
            ),
            (
                fourth_color.as_core(),
                parse_player_id(&fourth_player, "assignments[3].player")?,
            ),
        ];

        blocus_core::PlayerSlots::four_player(assignments)
            .map(Self::from_core)
            .map_err(map_input_error)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
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

#[pyclass(name = "GameConfig", frozen, from_py_object)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct GameConfig {
    inner: blocus_core::GameConfig,
}

impl GameConfig {
    pub const fn from_core(inner: blocus_core::GameConfig) -> Self {
        Self { inner }
    }

    pub const fn as_core(&self) -> blocus_core::GameConfig {
        self.inner
    }
}

#[pymethods]
impl GameConfig {
    #[new]
    fn new(
        game_id: &str,
        mode: &GameMode,
        scoring: &ScoringMode,
        turn_order: Vec<PlayerColor>,
        player_slots: &PlayerSlots,
    ) -> PyResult<Self> {
        let game_id = blocus_core::GameId::from_uuid(parse_uuid(game_id, "game_id")?);

        let [first, second, third, fourth]: [PlayerColor; 4] = turn_order
            .try_into()
            .map_err(|_values: Vec<PlayerColor>| crate::conversion::invalid_game_config_error())?;

        let turn_order = blocus_core::TurnOrder::try_new([
            first.as_core(),
            second.as_core(),
            third.as_core(),
            fourth.as_core(),
        ])
        .map_err(map_input_error)?;

        blocus_core::GameConfig::try_new(
            game_id,
            mode.as_core(),
            scoring.as_core(),
            turn_order,
            player_slots.as_core(),
        )
        .map(Self::from_core)
        .map_err(map_input_error)
    }

    #[staticmethod]
    fn two_player(
        game_id: &str,
        blue_red_player: &str,
        yellow_green_player: &str,
        scoring: &ScoringMode,
    ) -> PyResult<Self> {
        let slots = PlayerSlots::two_player(blue_red_player, yellow_green_player)?;
        Self::new(
            game_id,
            &GameMode::from_core(blocus_core::GameMode::TwoPlayer),
            scoring,
            vec![
                PlayerColor::from_core(blocus_core::PlayerColor::Blue),
                PlayerColor::from_core(blocus_core::PlayerColor::Yellow),
                PlayerColor::from_core(blocus_core::PlayerColor::Red),
                PlayerColor::from_core(blocus_core::PlayerColor::Green),
            ],
            &slots,
        )
    }

    #[staticmethod]
    fn four_player(
        game_id: &str,
        assignments: Vec<(PlayerColor, String)>,
        scoring: &ScoringMode,
        turn_order: Vec<PlayerColor>,
    ) -> PyResult<Self> {
        let slots = PlayerSlots::four_player(assignments)?;
        Self::new(
            game_id,
            &GameMode::from_core(blocus_core::GameMode::FourPlayer),
            scoring,
            turn_order,
            &slots,
        )
    }

    #[getter]
    fn game_id(&self) -> String {
        self.inner.game_id().to_string()
    }

    #[getter]
    fn mode(&self) -> GameMode {
        GameMode::from_core(self.inner.mode())
    }

    #[getter]
    fn scoring(&self) -> ScoringMode {
        ScoringMode::from_core(self.inner.scoring())
    }

    #[getter]
    fn turn_order(&self) -> Vec<PlayerColor> {
        self.inner
            .turn_order()
            .colors()
            .into_iter()
            .map(PlayerColor::from_core)
            .collect()
    }

    #[getter]
    fn player_slots(&self) -> PlayerSlots {
        PlayerSlots::from_core(self.inner.player_slots())
    }

    fn __repr__(&self) -> String {
        format!(
            "GameConfig(game_id={:?}, mode={:?}, scoring={:?})",
            self.game_id(),
            self.mode(),
            self.scoring()
        )
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
