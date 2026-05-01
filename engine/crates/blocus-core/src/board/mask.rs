//! Fixed-size board bit masks.

use crate::board::{BOARD_LANES, BoardIndex};
use crate::{BOARD_SIZE, ROW_STRIDE};

/// Mask containing all playable cells and no row-padding bits.
/// Mask containing the 20 playable cells of one padded 32-bit row.
pub const ROW_PLAYABLE_MASK: u128 = (1u128 << BOARD_SIZE) - 1;

/// Mask containing four playable padded rows inside one `u128` lane.
///
/// Each lane contains four 32-bit rows:
///
/// - row 0 at bits 0..31,
/// - row 1 at bits 32..63,
/// - row 2 at bits 64..95,
/// - row 3 at bits 96..127.
///
/// Only the lower 20 bits of each row are playable.
pub const LANE_PLAYABLE_MASK: u128 = ROW_PLAYABLE_MASK
    | (ROW_PLAYABLE_MASK << ROW_STRIDE)
    | (ROW_PLAYABLE_MASK << (ROW_STRIDE * 2))
    | (ROW_PLAYABLE_MASK << (ROW_STRIDE * 3));

/// Mask containing every playable board cell and no row-padding bits.
pub const PLAYABLE_MASK: BoardMask = BoardMask {
    lanes: [LANE_PLAYABLE_MASK; BOARD_LANES],
};

/// Fixed-size 640-bit board mask.
///
/// The mask uses five `u128` lanes because the internal board layout is
/// `20 rows × 32 bits = 640 bits`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct BoardMask {
    lanes: [u128; BOARD_LANES],
}

impl BoardMask {
    /// Empty board mask.
    pub const EMPTY: Self = Self {
        lanes: [0; BOARD_LANES],
    };

    /// Creates a board mask from raw lanes.
    ///
    /// This is intentionally public because white-box tests and later
    /// hash/serialization code need deterministic access to the lane layout.
    #[must_use]
    pub const fn from_lanes(lanes: [u128; BOARD_LANES]) -> Self {
        Self { lanes }
    }

    /// Returns the raw lanes.
    #[must_use]
    pub const fn lanes(self) -> [u128; BOARD_LANES] {
        self.lanes
    }

    /// Creates a one-bit mask for a board index.
    #[must_use]
    pub fn from_index(index: BoardIndex) -> Self {
        Self::EMPTY.inserted(index)
    }

    /// Returns whether this mask contains the index.
    #[must_use]
    pub fn contains(self, index: BoardIndex) -> bool {
        self.lanes[index.lane()] & index.lane_bit() != 0
    }

    /// Returns a copy of this mask with the index inserted.
    #[must_use]
    pub fn inserted(mut self, index: BoardIndex) -> Self {
        self.insert(index);
        self
    }

    /// Inserts the index into this mask.
    pub fn insert(&mut self, index: BoardIndex) {
        self.lanes[index.lane()] |= index.lane_bit();
    }

    /// Returns true if this mask intersects another mask.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        let mut lane = 0;

        while lane < BOARD_LANES {
            if self.lanes[lane] & other.lanes[lane] != 0 {
                return true;
            }

            lane += 1;
        }

        false
    }

    /// Returns the union of this mask and another mask.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        let mut lanes = [0; BOARD_LANES];
        let mut lane = 0;

        while lane < BOARD_LANES {
            lanes[lane] = self.lanes[lane] | other.lanes[lane];
            lane += 1;
        }

        Self { lanes }
    }

    /// Returns the cells in this mask that are not in another mask.
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        let mut lanes = [0; BOARD_LANES];
        let mut lane = 0;

        while lane < BOARD_LANES {
            lanes[lane] = self.lanes[lane] & !other.lanes[lane];
            lane += 1;
        }

        Self { lanes }
    }

    /// Returns true if the mask contains no bits.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        let mut lane = 0;

        while lane < BOARD_LANES {
            if self.lanes[lane] != 0 {
                return false;
            }

            lane += 1;
        }

        true
    }

    /// Returns true if every bit in this mask is also present in `other`.
    #[must_use]
    pub const fn is_subset_of(self, other: Self) -> bool {
        let mut lane = 0;

        while lane < BOARD_LANES {
            if self.lanes[lane] & !other.lanes[lane] != 0 {
                return false;
            }

            lane += 1;
        }

        true
    }

    /// Counts set bits.
    #[must_use]
    pub const fn count(self) -> u32 {
        let mut total = 0;
        let mut lane = 0;

        while lane < BOARD_LANES {
            total += self.lanes[lane].count_ones();
            lane += 1;
        }

        total
    }

    /// Returns true if all set bits are playable board cells.
    #[must_use]
    pub const fn is_playable_subset(self) -> bool {
        self.is_subset_of(PLAYABLE_MASK)
    }
}
