//! Board geometry constants.

/// Classic Blokus board width and height.
pub const BOARD_SIZE: u8 = 20;

/// Number of bits used for each padded board row.
pub const ROW_STRIDE: u8 = 32;

/// Number of padding bits at the end of each padded board row.
pub const ROW_PADDING_BITS: u8 = ROW_STRIDE - BOARD_SIZE;

/// Number of playable board cells.
pub const PLAYABLE_CELLS: usize = BOARD_SIZE as usize * BOARD_SIZE as usize;

/// Number of bits in the padded board representation.
pub const BOARD_BITS: usize = BOARD_SIZE as usize * ROW_STRIDE as usize;

/// Number of `u128` lanes required to store the padded board.
pub const BOARD_LANES: usize = BOARD_BITS / u128::BITS as usize;
