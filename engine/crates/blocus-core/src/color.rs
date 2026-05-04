//! Player colors.

use crate::error::InputError;
use core::fmt;

/// Number of colors in classic Blokus.
pub const CLASSIC_COLOR_COUNT: usize = 4;

/// Number of colors in Blokus Duo.
pub const DUO_COLOR_COUNT: usize = 2;

/// Maximum number of physical color slots supported by the engine.
pub const MAX_PLAYER_COLOR_COUNT: usize = 6;

/// Number of stored player-color slots.
///
/// This name is kept for compatibility with existing callers that construct
/// compact state arrays directly. Gameplay code should use
/// `GameMode::active_colors()` rather than iterating every storage slot.
pub const PLAYER_COLOR_COUNT: usize = MAX_PLAYER_COLOR_COUNT;

const CLASSIC_TURN_ORDER_LEN: u8 = 4;
const DUO_TURN_ORDER_LEN: u8 = 2;

/// One Blokus color identity.
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
    /// Black player color, used by Blokus Duo.
    Black,
    /// White player color, used by Blokus Duo.
    White,
}

impl PlayerColor {
    /// Classic colors in stable storage order.
    pub const CLASSIC: [Self; CLASSIC_COLOR_COUNT] =
        [Self::Blue, Self::Yellow, Self::Red, Self::Green];

    /// Duo colors in stable storage order.
    pub const DUO: [Self; DUO_COLOR_COUNT] = [Self::Black, Self::White];

    /// All supported colors in stable storage order.
    ///
    /// This order is used for fixed-size arrays, hashing, and indexing. It is
    /// not necessarily the gameplay turn order of every game.
    pub const ALL: [Self; MAX_PLAYER_COLOR_COUNT] = [
        Self::Blue,
        Self::Yellow,
        Self::Red,
        Self::Green,
        Self::Black,
        Self::White,
    ];

    /// Official clockwise order: blue, yellow, red, green.
    pub const OFFICIAL_FIXED_TURN_ORDER: [Self; CLASSIC_COLOR_COUNT] =
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
            Self::Black => 4,
            Self::White => 5,
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
            4 => Some(Self::Black),
            5 => Some(Self::White),
            _ => None,
        }
    }

    /// Returns whether this is a classic Blokus color.
    #[must_use]
    pub const fn is_classic(self) -> bool {
        matches!(self, Self::Blue | Self::Yellow | Self::Red | Self::Green)
    }

    /// Returns whether this is a Blokus Duo color.
    #[must_use]
    pub const fn is_duo(self) -> bool {
        matches!(self, Self::Black | Self::White)
    }

    /// Returns the bit associated with this color in compact color masks.
    #[must_use]
    pub const fn bit(self) -> u8 {
        1u8 << self.index()
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
            Self::Black => Self::White,
            Self::White => Self::Black,
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
            Self::Black => "black",
            Self::White => "white",
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

    /// The order must be black/white or white/black.
    ///
    /// This is the policy for Blokus Duo.
    DuoAlternating,
}

/// Game-specific cyclic turn order over active colors.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TurnOrder {
    colors: [PlayerColor; MAX_PLAYER_COLOR_COUNT],
    len: u8,
}

impl TurnOrder {
    /// Official fixed order: blue, yellow, red, green.
    pub const OFFICIAL_FIXED: Self = Self {
        colors: [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
            PlayerColor::Blue,
            PlayerColor::Blue,
        ],
        len: CLASSIC_TURN_ORDER_LEN,
    };

    /// Creates a turn order from a permutation of all four colors.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the order does not contain
    /// each color exactly once.
    pub const fn try_new(colors: [PlayerColor; CLASSIC_COLOR_COUNT]) -> Result<Self, InputError> {
        if contains_all_colors_once(colors) {
            Ok(Self {
                colors: [
                    colors[0],
                    colors[1],
                    colors[2],
                    colors[3],
                    PlayerColor::Blue,
                    PlayerColor::Blue,
                ],
                len: CLASSIC_TURN_ORDER_LEN,
            })
        } else {
            Err(InputError::InvalidGameConfig)
        }
    }

    /// Creates a Duo turn order from the first color.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] unless `first_color` is black
    /// or white.
    pub const fn duo(first_color: PlayerColor) -> Result<Self, InputError> {
        let second_color = match first_color {
            PlayerColor::Black => PlayerColor::White,
            PlayerColor::White => PlayerColor::Black,
            _ => return Err(InputError::InvalidGameConfig),
        };

        Ok(Self {
            colors: [
                first_color,
                second_color,
                PlayerColor::Blue,
                PlayerColor::Blue,
                PlayerColor::Blue,
                PlayerColor::Blue,
            ],
            len: DUO_TURN_ORDER_LEN,
        })
    }

    /// Returns the underlying color order.
    #[must_use]
    pub fn colors(self) -> Vec<PlayerColor> {
        self.colors[..usize::from(self.len)].to_vec()
    }

    /// Returns the active color order without allocation.
    #[must_use]
    pub fn colors_slice(&self) -> &[PlayerColor] {
        &self.colors[..usize::from(self.len)]
    }

    /// Returns the number of colors in the cyclic turn order.
    #[must_use]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    /// Returns true if the turn order contains no colors.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len == 0
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
            [
                self.colors[0],
                self.colors[1],
                self.colors[2],
                self.colors[3]
            ],
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
            [
                self.colors[0],
                self.colors[1],
                self.colors[2],
                self.colors[3]
            ],
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
        let mut position = 0usize;

        while position < self.len as usize {
            if self.colors[position].index() == target {
                return position;
            }

            position += 1;
        }

        0
    }

    /// Returns whether the active turn order contains this color.
    #[must_use]
    pub const fn contains(self, color: PlayerColor) -> bool {
        let mut position = 0usize;

        while position < self.len as usize {
            if self.colors[position].index() == color.index() {
                return true;
            }

            position += 1;
        }

        false
    }

    /// Returns the next color after `color` in this cyclic order.
    #[must_use]
    pub const fn next_after(self, color: PlayerColor) -> PlayerColor {
        let position = self.position_of(color);

        self.colors[(position + 1) % self.len()]
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
            TurnOrderPolicy::DuoAlternating => {
                if self.len == DUO_TURN_ORDER_LEN
                    && ((self.colors[0].index() == PlayerColor::Black.index()
                        && self.colors[1].index() == PlayerColor::White.index())
                        || (self.colors[0].index() == PlayerColor::White.index()
                            && self.colors[1].index() == PlayerColor::Black.index()))
                {
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

const fn contains_all_colors_once(colors: [PlayerColor; CLASSIC_COLOR_COUNT]) -> bool {
    let mut seen = [false; CLASSIC_COLOR_COUNT];
    let mut position = 0;

    while position < CLASSIC_COLOR_COUNT {
        let color_index = colors[position].index();

        if color_index >= CLASSIC_COLOR_COUNT {
            return false;
        }

        if seen[color_index] {
            return false;
        }

        seen[color_index] = true;
        position += 1;
    }

    true
}
