//! Game configuration and player/color assignment.

use crate::{
    GameId, InputError, PLAYER_COLOR_COUNT, PlayerColor, PlayerId, ScoringMode, TurnOrder,
    TurnOrderPolicy,
};

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
}

impl GameMode {
    /// Returns the required turn-order policy for this mode.
    #[must_use]
    pub const fn turn_order_policy(self) -> TurnOrderPolicy {
        match self {
            Self::TwoPlayer | Self::ThreePlayer => TurnOrderPolicy::OfficialFixed,
            Self::FourPlayer => TurnOrderPolicy::Flexible,
        }
    }

    /// Returns the number of participating players.
    #[must_use]
    pub const fn player_count(self) -> usize {
        match self {
            Self::TwoPlayer => 2,
            Self::ThreePlayer => 3,
            Self::FourPlayer => 4,
        }
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
    controllers: [Option<PlayerId>; PLAYER_COLOR_COUNT],
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
            controllers: [
                Some(blue_red_player),
                Some(yellow_green_player),
                Some(blue_red_player),
                Some(yellow_green_player),
            ],
            shared_color_turn: None,
        })
    }

    /// Creates a three-player assignment with one shared color.
    ///
    /// `owned_colors` must contain exactly three distinct colors controlled by
    /// exactly three distinct players. `shared_color_turn` must describe the
    /// remaining shared color and must alternate over exactly the same three
    /// players.
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
        let mut controllers = [None; PLAYER_COLOR_COUNT];

        for (color, player_id) in owned_colors {
            let index = color.index();

            if controllers[index].is_some() {
                return Err(InputError::InvalidGameConfig);
            }

            if color == shared_color_turn.color() {
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
        assignments: [(PlayerColor, PlayerId); PLAYER_COLOR_COUNT],
    ) -> Result<Self, InputError> {
        let mut controllers = [None; PLAYER_COLOR_COUNT];

        for (color, player_id) in assignments {
            let index = color.index();

            if controllers[index].is_some() {
                return Err(InputError::InvalidGameConfig);
            }

            controllers[index] = Some(player_id);
        }

        if has_duplicate_assigned_players(controllers) || controllers.contains(&None) {
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
    ///
    /// In three-player games, the shared color has no permanent controller and
    /// is represented as `None`.
    #[must_use]
    pub const fn controllers(self) -> [Option<PlayerId>; PLAYER_COLOR_COUNT] {
        self.controllers
    }

    /// Returns the permanent controller for a color.
    ///
    /// In three-player games this returns `None` for the shared color.
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
    ///
    /// For the three-player shared color, all players in the shared-color cycle
    /// can control it on their alternating shared turns.
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
    ///
    /// For normal colors this is the permanent controller. For a shared color,
    /// `shared_turn_index` chooses the alternating player from the shared-color
    /// cycle.
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
}

fn has_duplicate_assigned_players(controllers: [Option<PlayerId>; PLAYER_COLOR_COUNT]) -> bool {
    let mut outer = 0;

    while outer < PLAYER_COLOR_COUNT {
        let Some(player) = controllers[outer] else {
            outer += 1;
            continue;
        };

        let mut inner = outer + 1;

        while inner < PLAYER_COLOR_COUNT {
            if controllers[inner] == Some(player) {
                return true;
            }

            inner += 1;
        }

        outer += 1;
    }

    false
}
