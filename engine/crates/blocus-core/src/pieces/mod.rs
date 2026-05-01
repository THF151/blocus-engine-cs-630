//! Canonical Blokus piece repository.
//!
//! This module will contain piece definitions, shape bitmaps, orientation
//! generation, and the immutable standard 21-piece repository.

mod inventory;

pub use inventory::{ALL_PIECES_MASK, PieceInventory};
