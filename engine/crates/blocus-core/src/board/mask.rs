//! Fixed-size board bit masks.

use crate::board::{BOARD_LANES, BoardIndex, BoardSymmetry};
use crate::{BOARD_SIZE, InputError, ROW_STRIDE};

/// Mask containing the 20 playable cells of one padded 32-bit row.
pub const ROW_PLAYABLE_MASK: u128 = (1u128 << BOARD_SIZE) - 1;

/// Mask containing the cells in one padded row that have a valid east neighbor.
///
/// Columns `0..=18` may move east. Column `19` must be dropped.
const ROW_EAST_SHIFT_SOURCE_MASK: u128 = (1u128 << (BOARD_SIZE - 1)) - 1;

/// Mask containing the cells in one padded row that have a valid west neighbor.
///
/// Columns `1..=19` may move west. Column `0` must be dropped.
const ROW_WEST_SHIFT_SOURCE_MASK: u128 = ROW_PLAYABLE_MASK & !1u128;

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

/// Mask containing valid east-shift source cells for four padded rows inside
/// one `u128` lane.
const LANE_EAST_SHIFT_SOURCE_MASK: u128 = ROW_EAST_SHIFT_SOURCE_MASK
    | (ROW_EAST_SHIFT_SOURCE_MASK << ROW_STRIDE)
    | (ROW_EAST_SHIFT_SOURCE_MASK << (ROW_STRIDE * 2))
    | (ROW_EAST_SHIFT_SOURCE_MASK << (ROW_STRIDE * 3));

/// Mask containing valid west-shift source cells for four padded rows inside
/// one `u128` lane.
const LANE_WEST_SHIFT_SOURCE_MASK: u128 = ROW_WEST_SHIFT_SOURCE_MASK
    | (ROW_WEST_SHIFT_SOURCE_MASK << ROW_STRIDE)
    | (ROW_WEST_SHIFT_SOURCE_MASK << (ROW_STRIDE * 2))
    | (ROW_WEST_SHIFT_SOURCE_MASK << (ROW_STRIDE * 3));

/// Mask containing every playable board cell and no row-padding bits.
pub const PLAYABLE_MASK: BoardMask = BoardMask {
    lanes: [LANE_PLAYABLE_MASK; BOARD_LANES],
};

/// Mask containing every playable cell that has a valid east neighbor.
const EAST_SHIFT_SOURCE_MASK: BoardMask = BoardMask {
    lanes: [LANE_EAST_SHIFT_SOURCE_MASK; BOARD_LANES],
};

/// Mask containing every playable cell that has a valid west neighbor.
const WEST_SHIFT_SOURCE_MASK: BoardMask = BoardMask {
    lanes: [LANE_WEST_SHIFT_SOURCE_MASK; BOARD_LANES],
};

const ROW_SHIFT_BITS: u32 = ROW_STRIDE as u32;
const ROW_SHIFT_COMPLEMENT: u32 = u128::BITS - ROW_SHIFT_BITS;

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

    /// Creates a square playable mask from the top-left `size × size` cells of
    /// the fixed physical board.
    ///
    /// `size` must be at most the physical board size. Larger values produce
    /// the full physical playable mask.
    #[must_use]
    pub const fn square_playable_mask(size: u8) -> Self {
        let bounded_size = if size > BOARD_SIZE { BOARD_SIZE } else { size };
        let mut lanes = [0u128; BOARD_LANES];
        let mut row = 0u8;

        while row < bounded_size {
            let mut col = 0u8;

            while col < bounded_size {
                let bit_index = row as usize * ROW_STRIDE as usize + col as usize;
                let lane = bit_index / u128::BITS as usize;
                let offset = bit_index % u128::BITS as usize;
                lanes[lane] |= 1u128 << offset;
                col += 1;
            }

            row += 1;
        }

        Self { lanes }
    }

    /// Creates a board mask from raw lanes after validating that all set bits
    /// are playable board cells.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidBoardIndex`] if any set bit is outside the
    /// playable `20 × 20` board, including row-padding bits.
    pub const fn try_from_lanes(lanes: [u128; BOARD_LANES]) -> Result<Self, InputError> {
        let mask = Self { lanes };

        if mask.is_playable_subset() {
            Ok(mask)
        } else {
            Err(InputError::InvalidBoardIndex)
        }
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

    /// Returns all playable indices contained in this mask in ascending padded
    /// bit-index order.
    ///
    /// Invalid padding bits are ignored by the conversion step. Public API
    /// boundaries should prefer [`Self::try_from_lanes`] or state validation to
    /// reject invalid masks before this method is used.
    #[must_use]
    pub fn indices(self) -> Vec<BoardIndex> {
        let mut result = Vec::with_capacity(
            usize::try_from(self.count())
                .unwrap_or_else(|_| unreachable!("board mask count always fits in usize")),
        );

        for (lane_index, lane_value) in self.lanes.into_iter().enumerate() {
            let mut remaining = lane_value;

            while remaining != 0 {
                let offset = remaining.trailing_zeros();
                let bit_index = lane_index * u128::BITS as usize + offset as usize;

                if let Ok(raw_bit_index) = u16::try_from(bit_index)
                    && let Ok(index) = BoardIndex::from_bit_index(raw_bit_index)
                {
                    result.push(index);
                }

                remaining &= remaining - 1;
            }
        }

        result
    }

    /// Removes and returns the lowest playable index in this mask.
    #[must_use]
    pub fn pop_lowest_index(&mut self) -> Option<BoardIndex> {
        for lane_index in 0..BOARD_LANES {
            while self.lanes[lane_index] != 0 {
                let offset = self.lanes[lane_index].trailing_zeros();
                self.lanes[lane_index] &= self.lanes[lane_index] - 1;

                let bit_index = lane_index * u128::BITS as usize + offset as usize;
                let raw_bit_index = u16::try_from(bit_index)
                    .unwrap_or_else(|_| unreachable!("board bit index always fits in u16"));

                if let Ok(index) = BoardIndex::from_bit_index(raw_bit_index) {
                    return Some(index);
                }
            }
        }

        None
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

    /// Returns the intersection of this mask and another mask.
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        let mut lanes = [0; BOARD_LANES];
        let mut lane = 0;

        while lane < BOARD_LANES {
            lanes[lane] = self.lanes[lane] & other.lanes[lane];
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

    /// Returns all edge-adjacent neighbor cells of this mask.
    #[must_use]
    pub const fn edge_neighbors(self) -> Self {
        self.shift_north()
            .union(self.shift_south())
            .union(self.shift_east())
            .union(self.shift_west())
    }

    /// Returns all diagonal neighbor cells of this mask.
    #[must_use]
    pub const fn diagonal_neighbors(self) -> Self {
        let north = self.shift_north();
        let south = self.shift_south();

        north
            .shift_east()
            .union(north.shift_west())
            .union(south.shift_east())
            .union(south.shift_west())
    }

    /// Returns diagonal-only frontier cells for same-color corner contact.
    #[must_use]
    pub const fn diagonal_frontier(self) -> Self {
        self.diagonal_neighbors()
            .difference(self.edge_neighbors())
            .difference(self)
    }

    /// Returns this mask shifted by signed board coordinates.
    ///
    /// Positive row deltas shift south; positive column deltas shift east.
    #[must_use]
    pub fn shift_by(self, row_delta: i8, col_delta: i8) -> Self {
        let mut shifted = self;

        let mut row_steps = row_delta.unsigned_abs();
        while row_steps > 0 {
            shifted = if row_delta >= 0 {
                shifted.shift_south()
            } else {
                shifted.shift_north()
            };
            row_steps -= 1;
        }

        let mut col_steps = col_delta.unsigned_abs();
        while col_steps > 0 {
            shifted = if col_delta >= 0 {
                shifted.shift_east()
            } else {
                shifted.shift_west()
            };
            col_steps -= 1;
        }

        shifted
    }

    /// Returns this mask after applying a square-board symmetry.
    #[must_use]
    pub fn transformed(self, symmetry: BoardSymmetry) -> Self {
        symmetry.transform_mask(self)
    }

    /// Returns this mask shifted one row toward lower row indices.
    ///
    /// Bits in row 0 are dropped. This is a semantic board shift, not an
    /// unchecked raw integer shift.
    #[must_use]
    pub const fn shift_north(self) -> Self {
        let mut lanes = [0u128; BOARD_LANES];
        let mut lane = 0;

        while lane < BOARD_LANES {
            lanes[lane] = self.lanes[lane] >> ROW_SHIFT_BITS;

            if lane + 1 < BOARD_LANES {
                lanes[lane] |= self.lanes[lane + 1] << ROW_SHIFT_COMPLEMENT;
            }

            lane += 1;
        }

        Self { lanes }.intersection(PLAYABLE_MASK)
    }

    /// Returns this mask shifted one row toward higher row indices.
    ///
    /// Bits in row 19 are dropped. This is a semantic board shift, not an
    /// unchecked raw integer shift.
    #[must_use]
    pub const fn shift_south(self) -> Self {
        let mut lanes = [0u128; BOARD_LANES];
        let mut lane = BOARD_LANES;

        while lane > 0 {
            lane -= 1;
            lanes[lane] = self.lanes[lane] << ROW_SHIFT_BITS;

            if lane > 0 {
                lanes[lane] |= self.lanes[lane - 1] >> ROW_SHIFT_COMPLEMENT;
            }
        }

        Self { lanes }.intersection(PLAYABLE_MASK)
    }

    /// Returns this mask shifted one column toward lower column indices.
    ///
    /// Bits in column 0 are dropped instead of wrapping into the previous row's
    /// padding bits.
    #[must_use]
    pub const fn shift_west(self) -> Self {
        let source = self.intersection(WEST_SHIFT_SOURCE_MASK);
        let mut lanes = [0u128; BOARD_LANES];
        let mut lane = 0;

        while lane < BOARD_LANES {
            lanes[lane] = source.lanes[lane] >> 1;
            lane += 1;
        }

        Self { lanes }
    }

    /// Returns this mask shifted one column toward higher column indices.
    ///
    /// Bits in column 19 are dropped instead of moving into row-padding bits.
    #[must_use]
    pub const fn shift_east(self) -> Self {
        let source = self.intersection(EAST_SHIFT_SOURCE_MASK);
        let mut lanes = [0u128; BOARD_LANES];
        let mut lane = 0;

        while lane < BOARD_LANES {
            lanes[lane] = source.lanes[lane] << 1;
            lane += 1;
        }

        Self { lanes }
    }
}
