//! Compact canonical piece shapes and geometric transforms.

use crate::InputError;

/// Maximum local width or height of any official Blokus piece.
pub const MAX_SHAPE_EXTENT: u8 = 5;

/// Maximum number of occupied cells in any official Blokus piece.
pub const MAX_SHAPE_CELLS: usize = 5;

/// Shape rotation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Rotation {
    /// No rotation.
    Deg0,
    /// Clockwise quarter turn.
    Deg90,
    /// Half turn.
    Deg180,
    /// Counter-clockwise quarter turn.
    Deg270,
}

impl Rotation {
    /// All supported rotations in stable orientation-generation order.
    pub const ALL: [Self; 4] = [Self::Deg0, Self::Deg90, Self::Deg180, Self::Deg270];
}

/// Optional mirror transform.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Flip {
    /// No mirror transform.
    None,
    /// Mirror horizontally before rotation.
    Horizontal,
}

impl Flip {
    /// All supported flips in stable orientation-generation order.
    pub const ALL: [Self; 2] = [Self::None, Self::Horizontal];
}

/// Compact normalized shape bitmap.
///
/// The internal mask uses a fixed `5 × 5` local grid. Bit index is
/// `row * 5 + col`. The stored `width` and `height` describe the normalized
/// bounding box, while the mask may only contain bits inside that box.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ShapeBitmap {
    width: u8,
    height: u8,
    square_count: u8,
    cells: u32,
}

impl ShapeBitmap {
    /// Creates a normalized shape from occupied local cells.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the shape is empty, contains
    /// duplicate cells, or contains a cell outside the `5 × 5` local shape grid.
    pub fn from_cells(cells: &[(u8, u8)]) -> Result<Self, InputError> {
        if cells.is_empty() || cells.len() > MAX_SHAPE_CELLS {
            return Err(InputError::InvalidGameConfig);
        }

        let mut raw_mask = 0u32;
        let mut index = 0usize;

        while index < cells.len() {
            let (row, col) = cells[index];

            if row >= MAX_SHAPE_EXTENT || col >= MAX_SHAPE_EXTENT {
                return Err(InputError::InvalidGameConfig);
            }

            let bit = cell_bit(row, col);

            if raw_mask & bit != 0 {
                return Err(InputError::InvalidGameConfig);
            }

            raw_mask |= bit;
            index += 1;
        }

        Self::from_raw_mask(raw_mask)
    }

    /// Creates a normalized shape from a raw local `5 × 5` mask.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::InvalidGameConfig`] if the mask is empty or uses
    /// bits outside the `5 × 5` local shape grid.
    pub fn from_raw_mask(mask: u32) -> Result<Self, InputError> {
        if mask == 0 || mask & !local_shape_mask() != 0 {
            return Err(InputError::InvalidGameConfig);
        }

        let mut min_row = MAX_SHAPE_EXTENT;
        let mut min_col = MAX_SHAPE_EXTENT;
        let mut max_row = 0u8;
        let mut max_col = 0u8;
        let mut square_count = 0u8;

        let mut row = 0u8;
        while row < MAX_SHAPE_EXTENT {
            let mut col = 0u8;

            while col < MAX_SHAPE_EXTENT {
                if mask & cell_bit(row, col) != 0 {
                    min_row = min_row.min(row);
                    min_col = min_col.min(col);
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                    square_count += 1;
                }

                col += 1;
            }

            row += 1;
        }

        let mut normalized_mask = 0u32;

        row = min_row;
        while row <= max_row {
            let mut col = min_col;

            while col <= max_col {
                if mask & cell_bit(row, col) != 0 {
                    normalized_mask |= cell_bit(row - min_row, col - min_col);
                }

                if col == u8::MAX {
                    break;
                }
                col += 1;
            }

            if row == u8::MAX {
                break;
            }
            row += 1;
        }

        Ok(Self {
            width: max_col - min_col + 1,
            height: max_row - min_row + 1,
            square_count,
            cells: normalized_mask,
        })
    }

    /// Returns the normalized shape width.
    #[must_use]
    pub const fn width(self) -> u8 {
        self.width
    }

    /// Returns the normalized shape height.
    #[must_use]
    pub const fn height(self) -> u8 {
        self.height
    }

    /// Returns the number of occupied cells.
    #[must_use]
    pub const fn square_count(self) -> u8 {
        self.square_count
    }

    /// Returns the compact local `5 × 5` cell mask.
    #[must_use]
    pub const fn cell_mask(self) -> u32 {
        self.cells
    }

    /// Returns whether the normalized shape contains a local cell.
    #[must_use]
    pub fn contains(self, row: u8, col: u8) -> bool {
        row < MAX_SHAPE_EXTENT && col < MAX_SHAPE_EXTENT && self.cells & cell_bit(row, col) != 0
    }

    /// Returns occupied cells in row-major order.
    #[must_use]
    pub fn cells(self) -> Vec<(u8, u8)> {
        let mut result = Vec::with_capacity(usize::from(self.square_count));

        let mut row = 0u8;
        while row < self.height {
            let mut col = 0u8;

            while col < self.width {
                if self.contains(row, col) {
                    result.push((row, col));
                }

                col += 1;
            }

            row += 1;
        }

        result
    }

    /// Returns this shape after applying flip, rotation, and normalization.
    #[must_use]
    pub fn transformed(self, rotation: Rotation, flip: Flip) -> Self {
        let mut mask = 0u32;

        for (row, col) in self.cells() {
            let flipped_col = match flip {
                Flip::None => col,
                Flip::Horizontal => self.width - 1 - col,
            };

            let (next_row, next_col) = match rotation {
                Rotation::Deg0 => (row, flipped_col),
                Rotation::Deg90 => (flipped_col, self.height - 1 - row),
                Rotation::Deg180 => (self.height - 1 - row, self.width - 1 - flipped_col),
                Rotation::Deg270 => (self.width - 1 - flipped_col, row),
            };

            mask |= cell_bit(next_row, next_col);
        }

        Self::from_raw_mask(mask)
            .unwrap_or_else(|_| unreachable!("transforming a valid shape yields a valid shape"))
    }
}

const fn cell_bit(row: u8, col: u8) -> u32 {
    1u32 << (row as u32 * MAX_SHAPE_EXTENT as u32 + col as u32)
}

const fn local_shape_mask() -> u32 {
    (1u32 << (MAX_SHAPE_EXTENT as u32 * MAX_SHAPE_EXTENT as u32)) - 1
}
