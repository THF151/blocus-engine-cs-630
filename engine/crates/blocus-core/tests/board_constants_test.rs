use blocus_core::{
    BOARD_BITS, BOARD_LANES, BOARD_SIZE, PLAYABLE_CELLS, ROW_PADDING_BITS, ROW_STRIDE,
};

const CONST_BOARD_SIZE: u8 = BOARD_SIZE;
const CONST_ROW_STRIDE: u8 = ROW_STRIDE;
const CONST_ROW_PADDING_BITS: u8 = ROW_PADDING_BITS;
const CONST_PLAYABLE_CELLS: usize = PLAYABLE_CELLS;
const CONST_BOARD_BITS: usize = BOARD_BITS;
const CONST_BOARD_LANES: usize = BOARD_LANES;

#[test]
fn board_size_matches_classic_blokus_board() {
    assert_eq!(BOARD_SIZE, 20);
    assert_eq!(CONST_BOARD_SIZE, 20);
}

#[test]
fn row_stride_uses_padded_32_bit_rows() {
    assert_eq!(ROW_STRIDE, 32);
    assert_eq!(CONST_ROW_STRIDE, 32);
    assert_eq!(ROW_STRIDE > BOARD_SIZE, ROW_PADDING_BITS > 0);
}

#[test]
fn row_padding_bits_are_non_playable_tail_bits_per_row() {
    assert_eq!(ROW_PADDING_BITS, 12);
    assert_eq!(CONST_ROW_PADDING_BITS, 12);
    assert_eq!(ROW_PADDING_BITS, ROW_STRIDE - BOARD_SIZE);
}

#[test]
fn playable_cell_count_is_twenty_by_twenty() {
    assert_eq!(PLAYABLE_CELLS, 400);
    assert_eq!(CONST_PLAYABLE_CELLS, 400);
    assert_eq!(
        PLAYABLE_CELLS,
        usize::from(BOARD_SIZE) * usize::from(BOARD_SIZE)
    );
}

#[test]
fn padded_board_bit_count_is_twenty_rows_by_thirty_two_bits() {
    assert_eq!(BOARD_BITS, 640);
    assert_eq!(CONST_BOARD_BITS, 640);
    assert_eq!(
        BOARD_BITS,
        usize::from(BOARD_SIZE) * usize::from(ROW_STRIDE)
    );
}

#[test]
fn board_lane_count_fits_padded_board_into_five_u128_lanes() {
    assert_eq!(BOARD_LANES, 5);
    assert_eq!(CONST_BOARD_LANES, 5);
    assert_eq!(BOARD_LANES * u128::BITS as usize, BOARD_BITS);
}

#[test]
fn dense_twenty_by_twenty_board_would_not_fit_the_padded_layout() {
    let dense_bits = usize::from(BOARD_SIZE) * usize::from(BOARD_SIZE);

    assert_eq!(dense_bits, PLAYABLE_CELLS);
    assert!(BOARD_BITS > dense_bits);
    assert_eq!(
        BOARD_BITS - dense_bits,
        usize::from(BOARD_SIZE) * usize::from(ROW_PADDING_BITS)
    );
}

#[test]
fn board_layout_constants_are_self_consistent() {
    assert_eq!(ROW_STRIDE, BOARD_SIZE + ROW_PADDING_BITS);
    assert_eq!(BOARD_BITS % u128::BITS as usize, 0);
    assert_eq!(BOARD_BITS / u128::BITS as usize, BOARD_LANES);
    assert_eq!(
        PLAYABLE_CELLS + usize::from(BOARD_SIZE) * usize::from(ROW_PADDING_BITS),
        BOARD_BITS
    );
}

#[test]
fn row_stride_is_power_of_two_for_shift_friendly_indexing() {
    assert!(ROW_STRIDE.is_power_of_two());
}

#[test]
fn board_size_is_smaller_than_row_stride_so_horizontal_padding_exists() {
    assert_eq!(BOARD_SIZE < ROW_STRIDE, ROW_PADDING_BITS > 0);
}

#[test]
fn constants_are_usable_in_array_lengths() {
    let lanes = [0u128; BOARD_LANES];
    let playable_rows = [0u8; BOARD_SIZE as usize];

    assert_eq!(lanes.len(), 5);
    assert_eq!(playable_rows.len(), 20);
}
