//! Canonical Blokus piece repository.
//!
//! This module contains compact piece shapes, transform support, and the
//! immutable official 21-piece repository with unique orientations precomputed
//! exactly once.

mod inventory;
mod repository;
mod shape;

pub use inventory::{ALL_PIECES_MASK, PieceInventory};
pub use repository::{
    CanonicalPiece, MAX_UNIQUE_ORIENTATIONS, PieceOrientation, PieceRepository, standard_piece,
    standard_pieces, standard_repository,
};
pub use shape::{Flip, MAX_SHAPE_CELLS, MAX_SHAPE_EXTENT, Rotation, ShapeBitmap};
