//! Piece inventory tracking.

use crate::{PIECE_COUNT, PieceId};

/// Bit mask containing every official Blokus piece.
pub const ALL_PIECES_MASK: u32 = (1u32 << PIECE_COUNT) - 1;

/// Per-color piece inventory.
///
/// A set bit means the piece has already been used by that color.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct PieceInventory {
    used_mask: u32,
}

impl PieceInventory {
    /// Empty inventory: no piece has been used.
    pub const EMPTY: Self = Self { used_mask: 0 };

    /// Creates an inventory from a raw used-piece bit mask.
    ///
    /// Bits outside the official 21-piece range are ignored.
    #[must_use]
    pub const fn from_used_mask(used_mask: u32) -> Self {
        Self {
            used_mask: used_mask & ALL_PIECES_MASK,
        }
    }

    /// Returns the raw used-piece bit mask.
    #[must_use]
    pub const fn used_mask(self) -> u32 {
        self.used_mask
    }

    /// Returns the available-piece bit mask.
    #[must_use]
    pub const fn available_mask(self) -> u32 {
        ALL_PIECES_MASK & !self.used_mask
    }

    /// Returns true if the piece has already been used.
    #[must_use]
    pub const fn is_used(self, piece_id: PieceId) -> bool {
        self.used_mask & piece_id.inventory_bit() != 0
    }

    /// Returns true if the piece is still available.
    #[must_use]
    pub const fn is_available(self, piece_id: PieceId) -> bool {
        !self.is_used(piece_id)
    }

    /// Returns a copy with the piece marked as used.
    #[must_use]
    pub const fn marked_used(mut self, piece_id: PieceId) -> Self {
        self.mark_used(piece_id);
        self
    }

    /// Marks a piece as used.
    ///
    /// This operation is idempotent.
    pub const fn mark_used(&mut self, piece_id: PieceId) {
        self.used_mask |= piece_id.inventory_bit();
    }

    /// Returns the number of used pieces.
    #[must_use]
    pub const fn used_count(self) -> u32 {
        self.used_mask.count_ones()
    }

    /// Returns the number of available pieces.
    #[must_use]
    pub const fn available_count(self) -> u32 {
        PIECE_COUNT as u32 - self.used_count()
    }

    /// Returns true if all official pieces have been used.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.used_mask == ALL_PIECES_MASK
    }
}
