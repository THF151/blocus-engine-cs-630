//! Board primitives for the padded-row Blokus board.

mod constants;
mod index;
mod mask;
mod state;

pub use constants::{
    BOARD_BITS, BOARD_LANES, BOARD_SIZE, PLAYABLE_CELLS, ROW_PADDING_BITS, ROW_STRIDE,
};
pub use index::BoardIndex;
pub use mask::{BoardMask, PLAYABLE_MASK};
pub use state::BoardState;
