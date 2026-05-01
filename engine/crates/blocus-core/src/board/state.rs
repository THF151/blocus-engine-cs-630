//! Board occupancy state.

use crate::board::BoardMask;
use crate::color::{PLAYER_COLOR_COUNT, PlayerColor};

/// Occupancy state for all four Blokus colors.
///
/// The state stores one mask per color. Full-board occupancy is derived by
/// OR-ing the four masks instead of storing a redundant cached field.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct BoardState {
    occupied_by_color: [BoardMask; PLAYER_COLOR_COUNT],
}

impl BoardState {
    /// Empty board state.
    pub const EMPTY: Self = Self {
        occupied_by_color: [BoardMask::EMPTY; PLAYER_COLOR_COUNT],
    };

    /// Creates board state from per-color occupancy masks.
    #[must_use]
    pub const fn from_occupied_by_color(
        occupied_by_color: [BoardMask; PLAYER_COLOR_COUNT],
    ) -> Self {
        Self { occupied_by_color }
    }

    /// Returns all per-color masks.
    #[must_use]
    pub const fn occupied_by_color(self) -> [BoardMask; PLAYER_COLOR_COUNT] {
        self.occupied_by_color
    }

    /// Returns the occupancy mask for one color.
    #[must_use]
    pub const fn occupied(self, color: PlayerColor) -> BoardMask {
        self.occupied_by_color[color.index()]
    }

    /// Returns a mutable occupancy mask for one color.
    #[must_use]
    pub fn occupied_mut(&mut self, color: PlayerColor) -> &mut BoardMask {
        &mut self.occupied_by_color[color.index()]
    }

    /// Places a mask for one color.
    ///
    /// This method only mutates occupancy. Rule validation must happen before
    /// calling this method.
    pub fn place_mask(&mut self, color: PlayerColor, mask: BoardMask) {
        let index = color.index();
        self.occupied_by_color[index] = self.occupied_by_color[index].union(mask);
    }

    /// Returns occupancy across all colors.
    #[must_use]
    pub const fn occupied_all(self) -> BoardMask {
        let mut result = BoardMask::EMPTY;
        let mut index = 0;

        while index < PLAYER_COLOR_COUNT {
            result = result.union(self.occupied_by_color[index]);
            index += 1;
        }

        result
    }

    /// Returns true if no color has occupied cells.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.occupied_all().is_empty()
    }
}
