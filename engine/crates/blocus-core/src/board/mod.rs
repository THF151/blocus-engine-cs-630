//! Board geometry and bitboard primitives.

pub mod constants;

pub mod index;

pub use constants::{
    BOARD_BITS, BOARD_LANES, BOARD_SIZE, PLAYABLE_CELLS, ROW_PADDING_BITS, ROW_STRIDE,
};

pub use index::BoardIndex;
