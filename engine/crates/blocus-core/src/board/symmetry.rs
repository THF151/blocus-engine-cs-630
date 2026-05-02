//! Square-board geometric symmetries.

use crate::board::{BOARD_SIZE, BoardIndex, BoardMask};

/// One of the eight symmetries of a square board.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BoardSymmetry {
    /// Leave coordinates unchanged.
    Identity,
    /// Rotate 90 degrees clockwise.
    Rotate90,
    /// Rotate 180 degrees.
    Rotate180,
    /// Rotate 270 degrees clockwise.
    Rotate270,
    /// Mirror across the horizontal center line.
    ReflectHorizontal,
    /// Mirror across the vertical center line.
    ReflectVertical,
    /// Mirror across the main diagonal.
    ReflectMainDiagonal,
    /// Mirror across the anti-diagonal.
    ReflectAntiDiagonal,
}

impl BoardSymmetry {
    /// All board symmetries in stable order.
    pub const ALL: [Self; 8] = [
        Self::Identity,
        Self::Rotate90,
        Self::Rotate180,
        Self::Rotate270,
        Self::ReflectHorizontal,
        Self::ReflectVertical,
        Self::ReflectMainDiagonal,
        Self::ReflectAntiDiagonal,
    ];

    /// Returns the inverse transform.
    #[must_use]
    pub const fn inverse(self) -> Self {
        match self {
            Self::Identity
            | Self::Rotate180
            | Self::ReflectHorizontal
            | Self::ReflectVertical
            | Self::ReflectMainDiagonal
            | Self::ReflectAntiDiagonal => self,
            Self::Rotate90 => Self::Rotate270,
            Self::Rotate270 => Self::Rotate90,
        }
    }

    /// Transforms one board index.
    #[must_use]
    pub fn transform_index(self, index: BoardIndex) -> BoardIndex {
        let row = index.row();
        let col = index.col();
        let last = BOARD_SIZE - 1;

        let (next_row, next_col) = match self {
            Self::Identity => (row, col),
            Self::Rotate90 => (col, last - row),
            Self::Rotate180 => (last - row, last - col),
            Self::Rotate270 => (last - col, row),
            Self::ReflectHorizontal => (last - row, col),
            Self::ReflectVertical => (row, last - col),
            Self::ReflectMainDiagonal => (col, row),
            Self::ReflectAntiDiagonal => (last - col, last - row),
        };

        BoardIndex::from_row_col(next_row, next_col)
            .unwrap_or_else(|_| unreachable!("board symmetries preserve playable coordinates"))
    }

    /// Transforms every set cell in a mask.
    #[must_use]
    pub fn transform_mask(self, mask: BoardMask) -> BoardMask {
        let mut transformed = BoardMask::EMPTY;

        for index in mask.indices() {
            transformed.insert(self.transform_index(index));
        }

        transformed
    }
}
