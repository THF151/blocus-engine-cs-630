//! Player colors.

use crate::error::InputError;
use core::fmt;

/// Number of Blokus player colors.
pub const PLAYER_COLOR_COUNT: usize = 4;

/// One of the four Blokus colors.
///
/// Color identity is separate from turn order and board-corner assignment.
/// A color has a stable storage index, but game setup decides turn order and
/// starting-corner layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PlayerColor {
    /// Blue player color.
    Blue,
    /// Yellow player color.
    Yellow,
    /// Red player color.
    Red,
    /// Green player color.
    Green,
}

impl PlayerColor {
    /// All colors in stable storage order.
    ///
    /// This order is used for fixed-size arrays, hashing, and indexing. It is
    /// not necessarily the gameplay turn order of every game.
    pub const ALL: [Self; PLAYER_COLOR_COUNT] = [Self::Blue, Self::Yellow, Self::Red, Self::Green];

    /// Official clockwise order: blue, yellow, red, green.
    pub const OFFICIAL_FIXED_TURN_ORDER: [Self; PLAYER_COLOR_COUNT] =
        [Self::Blue, Self::Yellow, Self::Red, Self::Green];

    /// Returns this color's stable storage index.
    ///
    /// This is not a game-specific turn-order position.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Blue => 0,
            Self::Yellow => 1,
            Self::Red => 2,
            Self::Green => 3,
        }
    }

    /// Returns the color for a stable storage index.
    #[must_use]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Blue),
            1 => Some(Self::Yellow),
            2 => Some(Self::Red),
            3 => Some(Self::Green),
            _ => None,
        }
    }

    /// Returns the next color in the official clockwise order.
    ///
    /// Gameplay turn advancement should use [`TurnOrder::next_after`] because
    /// four-player games may rotate which color starts while preserving
    /// clockwise order.
    #[must_use]
    pub const fn next_in_official_fixed_order(self) -> Self {
        match self {
            Self::Blue => Self::Yellow,
            Self::Yellow => Self::Red,
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
        }
    }

    /// Returns the stable lowercase API name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Yellow => "yellow",
            Self::Red => "red",
            Self::Green => "green",
        }
    }
}

impl fmt::Display for PlayerColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Turn-order validation policy for a game variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TurnOrderPolicy {
    /// The order must be a clockwise rotation of blue, yellow, red, green.
    ///
    /// This is the policy for four-player games: the first player may vary,
    /// but play still proceeds clockwise around the board.
    ClockwiseRotation,

    /// The order must be blue, yellow, red, green.
    ///
    /// This is the policy for the official two-player and three-player
    /// variations.
    OfficialFixed,
}

/// Game-specific cyclic turn order over all four colors.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TurnOrder {
    colors: [PlayerColor; PLAYER_COLOR_COUNT],
}

impl TurnOrder {
    /// Official fixed order: blue, yellow, red, green.
    pub const OFFICIAL_FIXED: Self = Self {
        colors: PlayerColor::OFFICIAL_FIXED_TURN_ORDER,
    };

    /// Creates a turn order from a permutation of all four colors.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the order does not contain
    /// each color exactly once.
    pub const fn try_new(colors: [PlayerColor; PLAYER_COLOR_COUNT]) -> Result<Self, InputError> {
        if contains_all_colors_once(colors) {
            Ok(Self { colors })
        } else {
            Err(InputError::InvalidGameConfig)
        }
    }

    /// Returns the underlying color order.
    #[must_use]
    pub const fn colors(self) -> [PlayerColor; PLAYER_COLOR_COUNT] {
        self.colors
    }

    /// Returns the first color to act.
    #[must_use]
    pub const fn first(self) -> PlayerColor {
        self.colors[0]
    }

    /// Returns whether this order is the official fixed order.
    #[must_use]
    pub const fn is_official_fixed(self) -> bool {
        matches!(
            self.colors,
            [
                PlayerColor::Blue,
                PlayerColor::Yellow,
                PlayerColor::Red,
                PlayerColor::Green
            ]
        )
    }

    /// Returns whether this order preserves clockwise color progression.
    ///
    /// The first color may be any color, but the cycle must remain
    /// blue → yellow → red → green.
    #[must_use]
    pub const fn is_clockwise_rotation(self) -> bool {
        matches!(
            self.colors,
            [
                PlayerColor::Blue,
                PlayerColor::Yellow,
                PlayerColor::Red,
                PlayerColor::Green
            ] | [
                PlayerColor::Yellow,
                PlayerColor::Red,
                PlayerColor::Green,
                PlayerColor::Blue
            ] | [
                PlayerColor::Red,
                PlayerColor::Green,
                PlayerColor::Blue,
                PlayerColor::Yellow
            ] | [
                PlayerColor::Green,
                PlayerColor::Blue,
                PlayerColor::Yellow,
                PlayerColor::Red
            ]
        )
    }

    /// Returns the turn-order position of a color.
    ///
    /// Valid [`TurnOrder`] values always contain every [`PlayerColor`] exactly
    /// once, so every color has a position in the order.
    #[must_use]
    pub const fn position_of(self, color: PlayerColor) -> usize {
        let target = color.index();

        if self.colors[0].index() == target {
            0
        } else if self.colors[1].index() == target {
            1
        } else if self.colors[2].index() == target {
            2
        } else {
            3
        }
    }

    /// Returns the next color after `color` in this cyclic order.
    #[must_use]
    pub const fn next_after(self, color: PlayerColor) -> PlayerColor {
        let position = self.position_of(color);

        self.colors[(position + 1) % PLAYER_COLOR_COUNT]
    }

    /// Validates this turn order against a variant's policy.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the order violates the
    /// required policy.
    pub const fn validate_for_policy(self, policy: TurnOrderPolicy) -> Result<(), InputError> {
        match policy {
            TurnOrderPolicy::OfficialFixed => {
                if self.is_official_fixed() {
                    Ok(())
                } else {
                    Err(InputError::InvalidGameConfig)
                }
            }
            TurnOrderPolicy::ClockwiseRotation => {
                if self.is_clockwise_rotation() {
                    Ok(())
                } else {
                    Err(InputError::InvalidGameConfig)
                }
            }
        }
    }
}

impl Default for TurnOrder {
    fn default() -> Self {
        Self::OFFICIAL_FIXED
    }
}

const fn contains_all_colors_once(colors: [PlayerColor; PLAYER_COLOR_COUNT]) -> bool {
    let mut seen = [false; PLAYER_COLOR_COUNT];
    let mut position = 0;

    while position < PLAYER_COLOR_COUNT {
        let color_index = colors[position].index();

        if seen[color_index] {
            return false;
        }

        seen[color_index] = true;
        position += 1;
    }

    true
}
