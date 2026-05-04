//! Game configuration, player/color assignment, and turn state.

use crate::{
    BOARD_SIZE, BoardIndex, BoardMask, GameId, InputError, MAX_PLAYER_COLOR_COUNT, PlayerColor,
    PlayerId, ScoringMode, TurnOrder, TurnOrderPolicy,
};

/// Blokus Duo visible board size.
pub const DUO_BOARD_SIZE: u8 = 14;

/// First zero-based Duo starting point.
pub const DUO_START_A: (u8, u8) = (4, 4);

/// Second zero-based Duo starting point.
pub const DUO_START_B: (u8, u8) = (9, 9);

/// Supported Blokus game modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum GameMode {
    /// Two players: one controls blue/red, the other controls yellow/green.
    TwoPlayer,
    /// Three players: three colors are individually owned and the fourth color
    /// is shared.
    ThreePlayer,
    /// Four players: each player controls exactly one color.
    FourPlayer,
    /// Blokus Duo: two players, black and white, on a 14×14 board.
    Duo,
}

impl GameMode {
    /// Returns the required turn-order policy for this mode.
    #[must_use]
    pub const fn turn_order_policy(self) -> TurnOrderPolicy {
        match self {
            Self::TwoPlayer | Self::ThreePlayer => TurnOrderPolicy::OfficialFixed,
            Self::FourPlayer => TurnOrderPolicy::ClockwiseRotation,
            Self::Duo => TurnOrderPolicy::DuoAlternating,
        }
    }

    /// Returns the number of participating players.
    #[must_use]
    pub const fn player_count(self) -> usize {
        match self {
            Self::TwoPlayer | Self::Duo => 2,
            Self::ThreePlayer => 3,
            Self::FourPlayer => 4,
        }
    }

    /// Returns the visible board size for this mode.
    #[must_use]
    pub const fn board_size(self) -> u8 {
        match self {
            Self::TwoPlayer | Self::ThreePlayer | Self::FourPlayer => BOARD_SIZE,
            Self::Duo => DUO_BOARD_SIZE,
        }
    }

    /// Returns the active colors for this mode in stable storage order.
    #[must_use]
    pub const fn active_colors(self) -> &'static [PlayerColor] {
        match self {
            Self::TwoPlayer | Self::ThreePlayer | Self::FourPlayer => &PlayerColor::CLASSIC,
            Self::Duo => &PlayerColor::DUO,
        }
    }

    /// Returns true if `color` participates in this mode.
    #[must_use]
    pub fn is_active_color(self, color: PlayerColor) -> bool {
        self.active_colors().contains(&color)
    }

    /// Returns the active-color bit mask for this mode.
    #[must_use]
    pub const fn active_color_bits(self) -> u8 {
        match self {
            Self::TwoPlayer | Self::ThreePlayer | Self::FourPlayer => {
                PlayerColor::Blue.bit()
                    | PlayerColor::Yellow.bit()
                    | PlayerColor::Red.bit()
                    | PlayerColor::Green.bit()
            }
            Self::Duo => PlayerColor::Black.bit() | PlayerColor::White.bit(),
        }
    }

    /// Returns the derived ruleset for this mode.
    #[must_use]
    pub const fn ruleset(self) -> Ruleset {
        Ruleset::for_mode(self)
    }
}

/// Logical board geometry for a mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BoardGeometry {
    size: u8,
    playable_mask: BoardMask,
}

impl BoardGeometry {
    /// Creates a square geometry backed by the fixed physical board.
    #[must_use]
    pub const fn square(size: u8) -> Self {
        Self {
            size,
            playable_mask: BoardMask::square_playable_mask(size),
        }
    }

    /// Classic 20×20 geometry.
    #[must_use]
    pub const fn classic() -> Self {
        Self::square(BOARD_SIZE)
    }

    /// Duo 14×14 geometry.
    #[must_use]
    pub const fn duo() -> Self {
        Self::square(DUO_BOARD_SIZE)
    }

    /// Returns the visible board size.
    #[must_use]
    pub const fn size(self) -> u8 {
        self.size
    }

    /// Returns the playable-cell mask for this geometry.
    #[must_use]
    pub const fn playable_mask(self) -> BoardMask {
        self.playable_mask
    }
}

/// Opening placement policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum OpeningPolicy {
    /// Classic fixed-corner starts.
    ClassicCorners,
    /// Duo's two shared starting points.
    DuoStartingPoints {
        /// First starting point.
        first: BoardIndex,
        /// Second starting point.
        second: BoardIndex,
    },
}

/// Compact derived ruleset for a mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Ruleset {
    mode: GameMode,
    geometry: BoardGeometry,
    opening_policy: OpeningPolicy,
}

impl Ruleset {
    /// Returns the ruleset for a game mode.
    ///
    /// # Panics
    ///
    /// Panics only if the compile-time Duo starting coordinates are outside
    /// the fixed physical board, which would indicate a broken engine
    /// constant.
    #[must_use]
    pub const fn for_mode(mode: GameMode) -> Self {
        match mode {
            GameMode::TwoPlayer | GameMode::ThreePlayer | GameMode::FourPlayer => Self {
                mode,
                geometry: BoardGeometry::classic(),
                opening_policy: OpeningPolicy::ClassicCorners,
            },
            GameMode::Duo => Self {
                mode,
                geometry: BoardGeometry::duo(),
                opening_policy: OpeningPolicy::DuoStartingPoints {
                    first: match BoardIndex::from_row_col(DUO_START_A.0, DUO_START_A.1) {
                        Ok(index) => index,
                        Err(_) => panic!("configured Duo start A must be on the physical board"),
                    },
                    second: match BoardIndex::from_row_col(DUO_START_B.0, DUO_START_B.1) {
                        Ok(index) => index,
                        Err(_) => panic!("configured Duo start B must be on the physical board"),
                    },
                },
            },
        }
    }

    /// Returns the mode.
    #[must_use]
    pub const fn mode(self) -> GameMode {
        self.mode
    }

    /// Returns the logical board geometry.
    #[must_use]
    pub const fn geometry(self) -> BoardGeometry {
        self.geometry
    }

    /// Returns the opening policy.
    #[must_use]
    pub const fn opening_policy(self) -> OpeningPolicy {
        self.opening_policy
    }
}

/// Alternating ownership of the shared color in a three-player game.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SharedColorTurn {
    color: PlayerColor,
    players: [PlayerId; 3],
}

impl SharedColorTurn {
    /// Creates the shared-color turn ownership cycle.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the player cycle contains
    /// duplicate players.
    pub fn try_new(color: PlayerColor, players: [PlayerId; 3]) -> Result<Self, InputError> {
        if players[0] == players[1] || players[0] == players[2] || players[1] == players[2] {
            Err(InputError::InvalidGameConfig)
        } else {
            Ok(Self { color, players })
        }
    }

    /// Returns the shared color.
    #[must_use]
    pub const fn color(self) -> PlayerColor {
        self.color
    }

    /// Returns the alternating player cycle.
    #[must_use]
    pub const fn players(self) -> [PlayerId; 3] {
        self.players
    }

    /// Returns the owner of the shared color for the given shared-color turn.
    #[must_use]
    pub const fn owner_at(self, shared_turn_index: usize) -> PlayerId {
        self.players[shared_turn_index % 3]
    }

    /// Returns true if the player participates in the shared-color cycle.
    #[must_use]
    pub fn contains_player(self, player_id: PlayerId) -> bool {
        self.players.contains(&player_id)
    }
}

/// Validated color ownership for a game.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PlayerSlots {
    mode: GameMode,
    controllers: [Option<PlayerId>; MAX_PLAYER_COLOR_COUNT],
    shared_color_turn: Option<SharedColorTurn>,
}

impl PlayerSlots {
    /// Creates the official two-player assignment:
    ///
    /// - player one controls blue and red
    /// - player two controls yellow and green
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if both slots use the same
    /// player.
    pub fn two_player(
        blue_red_player: PlayerId,
        yellow_green_player: PlayerId,
    ) -> Result<Self, InputError> {
        if blue_red_player == yellow_green_player {
            return Err(InputError::InvalidGameConfig);
        }

        Ok(Self {
            mode: GameMode::TwoPlayer,
            controllers: {
                let mut controllers = [None; MAX_PLAYER_COLOR_COUNT];
                controllers[PlayerColor::Blue.index()] = Some(blue_red_player);
                controllers[PlayerColor::Yellow.index()] = Some(yellow_green_player);
                controllers[PlayerColor::Red.index()] = Some(blue_red_player);
                controllers[PlayerColor::Green.index()] = Some(yellow_green_player);
                controllers
            },
            shared_color_turn: None,
        })
    }

    /// Creates a Blokus Duo assignment.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if both colors use the same
    /// player.
    pub fn duo(black_player: PlayerId, white_player: PlayerId) -> Result<Self, InputError> {
        if black_player == white_player {
            return Err(InputError::InvalidGameConfig);
        }

        let mut controllers = [None; MAX_PLAYER_COLOR_COUNT];
        controllers[PlayerColor::Black.index()] = Some(black_player);
        controllers[PlayerColor::White.index()] = Some(white_player);

        Ok(Self {
            mode: GameMode::Duo,
            controllers,
            shared_color_turn: None,
        })
    }

    /// Creates a three-player assignment with one shared color.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if colors or players are
    /// duplicated, if the shared color is also individually owned, or if the
    /// shared-color turn cycle does not contain the same three players.
    pub fn three_player(
        owned_colors: [(PlayerColor, PlayerId); 3],
        shared_color_turn: SharedColorTurn,
    ) -> Result<Self, InputError> {
        let mut controllers = [None; MAX_PLAYER_COLOR_COUNT];

        for (color, player_id) in owned_colors {
            if !color.is_classic() || !shared_color_turn.color().is_classic() {
                return Err(InputError::InvalidGameConfig);
            }

            let index = color.index();

            if controllers[index].is_some() || color == shared_color_turn.color() {
                return Err(InputError::InvalidGameConfig);
            }

            controllers[index] = Some(player_id);
        }

        if has_duplicate_assigned_players(controllers) {
            return Err(InputError::InvalidGameConfig);
        }

        if controllers[shared_color_turn.color().index()].is_some() {
            return Err(InputError::InvalidGameConfig);
        }

        for player_id in shared_color_turn.players() {
            if !controllers.contains(&Some(player_id)) {
                return Err(InputError::InvalidGameConfig);
            }
        }

        Ok(Self {
            mode: GameMode::ThreePlayer,
            controllers,
            shared_color_turn: Some(shared_color_turn),
        })
    }

    /// Creates a four-player assignment.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if colors or players are
    /// duplicated.
    pub fn four_player(
        assignments: [(PlayerColor, PlayerId); crate::CLASSIC_COLOR_COUNT],
    ) -> Result<Self, InputError> {
        let mut controllers = [None; MAX_PLAYER_COLOR_COUNT];

        for (color, player_id) in assignments {
            if !color.is_classic() {
                return Err(InputError::InvalidGameConfig);
            }

            let index = color.index();

            if controllers[index].is_some() {
                return Err(InputError::InvalidGameConfig);
            }

            controllers[index] = Some(player_id);
        }

        if has_duplicate_assigned_players(controllers)
            || PlayerColor::CLASSIC
                .into_iter()
                .any(|color| controllers[color.index()].is_none())
        {
            return Err(InputError::InvalidGameConfig);
        }

        Ok(Self {
            mode: GameMode::FourPlayer,
            controllers,
            shared_color_turn: None,
        })
    }

    /// Returns the game mode.
    #[must_use]
    pub const fn mode(self) -> GameMode {
        self.mode
    }

    /// Returns all permanent color controllers.
    #[must_use]
    pub const fn controllers(self) -> [Option<PlayerId>; MAX_PLAYER_COLOR_COUNT] {
        self.controllers
    }

    /// Returns the permanent controller for a color.
    #[must_use]
    pub const fn controller_of(self, color: PlayerColor) -> Option<PlayerId> {
        self.controllers[color.index()]
    }

    /// Returns the shared-color turn cycle, if this is a three-player game.
    #[must_use]
    pub const fn shared_color_turn(self) -> Option<SharedColorTurn> {
        self.shared_color_turn
    }

    /// Returns the shared color, if this is a three-player game.
    #[must_use]
    pub const fn shared_color(self) -> Option<PlayerColor> {
        match self.shared_color_turn {
            Some(shared) => Some(shared.color()),
            None => None,
        }
    }

    /// Returns true if the player can ever control the color.
    #[must_use]
    pub fn can_control_color(self, player_id: PlayerId, color: PlayerColor) -> bool {
        if self.controller_of(color) == Some(player_id) {
            return true;
        }

        match self.shared_color_turn {
            Some(shared) => shared.color() == color && shared.contains_player(player_id),
            None => false,
        }
    }

    /// Returns the player who controls a color for a specific turn context.
    #[must_use]
    pub const fn turn_controller_of(
        self,
        color: PlayerColor,
        shared_turn_index: usize,
    ) -> Option<PlayerId> {
        match self.shared_color_turn {
            Some(shared) if shared.color().index() == color.index() => {
                Some(shared.owner_at(shared_turn_index))
            }
            _ => self.controller_of(color),
        }
    }
}

/// Turn progression state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TurnState {
    current_color: PlayerColor,
    passed_mask: u8,
    shared_color_turn_index: usize,
}

impl TurnState {
    /// Creates turn state from the first color in a turn order.
    #[must_use]
    pub const fn new(turn_order: TurnOrder) -> Self {
        Self {
            current_color: turn_order.first(),
            passed_mask: 0,
            shared_color_turn_index: 0,
        }
    }

    /// Creates turn state from raw validated fields.
    #[must_use]
    pub const fn from_parts(
        current_color: PlayerColor,
        passed_mask: u8,
        shared_color_turn_index: usize,
    ) -> Self {
        Self {
            current_color,
            passed_mask: passed_mask & all_storage_color_bits(),
            shared_color_turn_index,
        }
    }

    /// Returns the current color.
    #[must_use]
    pub const fn current_color(self) -> PlayerColor {
        self.current_color
    }

    /// Returns the passed-color bit mask.
    #[must_use]
    pub const fn passed_mask(self) -> u8 {
        self.passed_mask
    }

    /// Returns the shared-color turn index.
    #[must_use]
    pub const fn shared_color_turn_index(self) -> usize {
        self.shared_color_turn_index
    }

    /// Returns true if the color has permanently passed.
    #[must_use]
    pub const fn is_passed(self, color: PlayerColor) -> bool {
        self.passed_mask & color_bit(color) != 0
    }

    /// Marks a color as passed.
    pub const fn mark_passed(&mut self, color: PlayerColor) {
        self.passed_mask |= color_bit(color);
    }

    /// Returns a copy with a color marked as passed.
    #[must_use]
    pub const fn marked_passed(mut self, color: PlayerColor) -> Self {
        self.mark_passed(color);
        self
    }

    /// Returns true if every color has passed.
    #[must_use]
    pub const fn all_colors_passed(self) -> bool {
        self.passed_mask & classic_color_bits() == classic_color_bits()
    }

    /// Returns true if every active color for `mode` has passed.
    #[must_use]
    pub const fn all_active_colors_passed(self, mode: GameMode) -> bool {
        self.passed_mask & mode.active_color_bits() == mode.active_color_bits()
    }

    /// Returns the number of colors that have passed.
    #[must_use]
    pub const fn passed_count(self) -> u32 {
        self.passed_mask.count_ones()
    }

    /// Returns the active player for the current color.
    #[must_use]
    pub const fn current_player(self, player_slots: PlayerSlots) -> Option<PlayerId> {
        player_slots.turn_controller_of(self.current_color, self.shared_color_turn_index)
    }

    /// Returns the active player for the current color.
    #[must_use]
    pub const fn active_controller(self, player_slots: PlayerSlots) -> Option<PlayerId> {
        self.current_player(player_slots)
    }

    /// Returns true if `player_id` is the player scheduled to act now.
    #[must_use]
    pub const fn is_active_controller(
        self,
        player_slots: PlayerSlots,
        player_id: PlayerId,
    ) -> bool {
        match self.active_controller(player_slots) {
            Some(active_player) => {
                active_player.as_uuid().as_u128() == player_id.as_uuid().as_u128()
            }
            None => false,
        }
    }

    /// Advances to the next non-passed color.
    ///
    /// Returns `None` if all colors have passed. When the current color is the
    /// shared color in a three-player game, the shared turn index advances
    /// before the next turn is selected.
    pub fn advance(&mut self, turn_order: TurnOrder, player_slots: PlayerSlots) -> Option<Self> {
        if player_slots.shared_color() == Some(self.current_color) {
            self.shared_color_turn_index = self.shared_color_turn_index.saturating_add(1);
        }

        if self.all_active_colors_passed(player_slots.mode()) {
            return None;
        }

        let mut next_color = turn_order.next_after(self.current_color);
        let mut checked = 0usize;

        while checked < turn_order.len() {
            if !self.is_passed(next_color) {
                self.current_color = next_color;
                return Some(*self);
            }

            next_color = turn_order.next_after(next_color);
            checked += 1;
        }

        None
    }
}

/// Validated game configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct GameConfig {
    game_id: GameId,
    mode: GameMode,
    scoring: ScoringMode,
    turn_order: TurnOrder,
    player_slots: PlayerSlots,
}

impl GameConfig {
    /// Creates a validated game configuration.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the player slots do not
    /// match the mode or if the turn order violates the mode policy.
    pub fn try_new(
        game_id: GameId,
        mode: GameMode,
        scoring: ScoringMode,
        turn_order: TurnOrder,
        player_slots: PlayerSlots,
    ) -> Result<Self, InputError> {
        if player_slots.mode() != mode {
            return Err(InputError::InvalidGameConfig);
        }

        if mode == GameMode::Duo && scoring != ScoringMode::Advanced {
            return Err(InputError::InvalidGameConfig);
        }

        turn_order.validate_for_policy(mode.turn_order_policy())?;

        Ok(Self {
            game_id,
            mode,
            scoring,
            turn_order,
            player_slots,
        })
    }

    /// Returns the game identifier.
    #[must_use]
    pub const fn game_id(self) -> GameId {
        self.game_id
    }

    /// Returns the game mode.
    #[must_use]
    pub const fn mode(self) -> GameMode {
        self.mode
    }

    /// Returns the scoring mode.
    #[must_use]
    pub const fn scoring(self) -> ScoringMode {
        self.scoring
    }

    /// Returns the game-specific turn order.
    #[must_use]
    pub const fn turn_order(self) -> TurnOrder {
        self.turn_order
    }

    /// Returns validated player slots.
    #[must_use]
    pub const fn player_slots(self) -> PlayerSlots {
        self.player_slots
    }

    /// Creates a Blokus Duo configuration.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if player assignment or first
    /// color is invalid.
    pub fn duo(
        game_id: GameId,
        black_player: PlayerId,
        white_player: PlayerId,
        first_color: PlayerColor,
    ) -> Result<Self, InputError> {
        Self::try_new(
            game_id,
            GameMode::Duo,
            ScoringMode::Advanced,
            TurnOrder::duo(first_color)?,
            PlayerSlots::duo(black_player, white_player)?,
        )
    }
}

const fn color_bit(color: PlayerColor) -> u8 {
    color.bit()
}

const fn classic_color_bits() -> u8 {
    color_bit(PlayerColor::Blue)
        | color_bit(PlayerColor::Yellow)
        | color_bit(PlayerColor::Red)
        | color_bit(PlayerColor::Green)
}

const fn all_storage_color_bits() -> u8 {
    classic_color_bits() | color_bit(PlayerColor::Black) | color_bit(PlayerColor::White)
}

fn has_duplicate_assigned_players(controllers: [Option<PlayerId>; MAX_PLAYER_COLOR_COUNT]) -> bool {
    let mut outer = 0;

    while outer < MAX_PLAYER_COLOR_COUNT {
        let Some(player) = controllers[outer] else {
            outer += 1;
            continue;
        };

        let mut inner = outer + 1;

        while inner < MAX_PLAYER_COLOR_COUNT {
            if controllers[inner] == Some(player) {
                return true;
            }

            inner += 1;
        }

        outer += 1;
    }

    false
}
