//! Validated playable board indices.

use crate::board::{BOARD_BITS, BOARD_SIZE, ROW_STRIDE};
use crate::error::InputError;
use core::fmt;

const BOARD_SIZE_U16: u16 = BOARD_SIZE as u16;
const ROW_STRIDE_U16: u16 = ROW_STRIDE as u16;
const LANE_BITS: usize = 128;
const LANE_BITS_U16: u16 = 128;

/// Validated playable board cell index.
///
/// Internally this uses padded-row indexing:
///
/// `bit_index = row * 32 + col`.
///
/// Only cells where `row < 20` and `col < 20` are valid. Padding bits in
/// columns `20..32` are rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BoardIndex(u16);

impl BoardIndex {
    /// Creates a playable board index from a row and column.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidBoardIndex`] if `row >= 20` or `col >= 20`.
    pub const fn from_row_col(row: u8, col: u8) -> Result<Self, InputError> {
        if row < BOARD_SIZE && col < BOARD_SIZE {
            Ok(Self(row as u16 * ROW_STRIDE_U16 + col as u16))
        } else {
            Err(InputError::InvalidBoardIndex)
        }
    }

    /// Creates a playable board index from a padded bit index.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidBoardIndex`] if the bit index is outside
    /// the padded board or points at a row-padding bit.
    pub const fn from_bit_index(bit_index: u16) -> Result<Self, InputError> {
        if bit_index as usize >= BOARD_BITS {
            return Err(InputError::InvalidBoardIndex);
        }

        let row = bit_index / ROW_STRIDE_U16;
        let col = bit_index % ROW_STRIDE_U16;

        if row < BOARD_SIZE_U16 && col < BOARD_SIZE_U16 {
            Ok(Self(bit_index))
        } else {
            Err(InputError::InvalidBoardIndex)
        }
    }

    /// Returns the row.
    #[must_use]
    pub fn row(self) -> u8 {
        u8::try_from(self.0 / ROW_STRIDE_U16)
            .unwrap_or_else(|_| unreachable!("validated board row always fits in u8"))
    }

    /// Returns the column.
    #[must_use]
    pub fn col(self) -> u8 {
        u8::try_from(self.0 % ROW_STRIDE_U16)
            .unwrap_or_else(|_| unreachable!("validated board column always fits in u8"))
    }

    /// Returns the padded bit index.
    #[must_use]
    pub const fn bit_index(self) -> u16 {
        self.0
    }

    /// Returns the `u128` lane containing this bit.
    #[must_use]
    pub const fn lane(self) -> usize {
        self.0 as usize / LANE_BITS
    }

    /// Returns the bit offset inside the containing `u128` lane.
    #[must_use]
    pub fn offset(self) -> u32 {
        u32::from(self.0 % LANE_BITS_U16)
    }

    /// Returns a one-bit mask for this index inside its lane.
    #[must_use]
    pub fn lane_bit(self) -> u128 {
        1u128 << self.offset()
    }
}

impl TryFrom<u16> for BoardIndex {
    type Error = InputError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::from_bit_index(value)
    }
}

impl From<BoardIndex> for u16 {
    fn from(value: BoardIndex) -> Self {
        value.0
    }
}

impl fmt::Display for BoardIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
