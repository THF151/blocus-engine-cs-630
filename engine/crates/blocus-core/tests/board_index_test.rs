use blocus_core::{BOARD_BITS, BOARD_SIZE, BoardIndex, InputError, ROW_STRIDE};
use std::collections::HashSet;

const FIRST_BIT_AFTER_PADDED_BOARD: u16 = 640;

const CONST_TOP_LEFT: BoardIndex = match BoardIndex::from_row_col(0, 0) {
    Ok(index) => index,
    Err(_) => panic!("top-left should be valid in const context"),
};

const CONST_TOP_RIGHT: BoardIndex = match BoardIndex::from_row_col(0, BOARD_SIZE - 1) {
    Ok(index) => index,
    Err(_) => panic!("top-right should be valid in const context"),
};

const CONST_BOTTOM_RIGHT: BoardIndex =
    match BoardIndex::from_row_col(BOARD_SIZE - 1, BOARD_SIZE - 1) {
        Ok(index) => index,
        Err(_) => panic!("bottom-right should be valid in const context"),
    };

const CONST_TOP_LEFT_FROM_BIT: BoardIndex = match BoardIndex::from_bit_index(0) {
    Ok(index) => index,
    Err(_) => panic!("bit index 0 should be valid in const context"),
};

const CONST_TOP_LEFT_BIT_INDEX: u16 = CONST_TOP_LEFT.bit_index();
const CONST_TOP_LEFT_LANE: usize = CONST_TOP_LEFT.lane();

fn assert_index(row: u8, col: u8, bit_index: u16, lane: usize, offset: u32) {
    let Ok(index) = BoardIndex::from_row_col(row, col) else {
        panic!("row {row}, col {col} should be valid");
    };

    assert_eq!(index.row(), row);
    assert_eq!(index.col(), col);
    assert_eq!(index.bit_index(), bit_index);
    assert_eq!(index.lane(), lane);
    assert_eq!(index.offset(), offset);
    assert_eq!(index.lane_bit(), 1u128 << offset);
    assert_eq!(index.to_string(), bit_index.to_string());

    let Ok(from_bit_index) = BoardIndex::from_bit_index(bit_index) else {
        panic!("bit index {bit_index} should be valid");
    };

    assert_eq!(from_bit_index, index);
    assert_eq!(BoardIndex::try_from(bit_index), Ok(index));

    let raw: u16 = index.into();
    assert_eq!(raw, bit_index);
}

#[test]
fn board_index_maps_row_col_to_padded_bit_index() {
    assert_index(0, 0, 0, 0, 0);
    assert_index(0, 19, 19, 0, 19);
    assert_index(1, 0, 32, 0, 32);
    assert_index(3, 31_u8.saturating_sub(12), 115, 0, 115);
    assert_index(4, 0, 128, 1, 0);
    assert_index(19, 0, 608, 4, 96);
    assert_index(19, 19, 627, 4, 115);
}

#[test]
fn board_index_rejects_out_of_bounds_row_or_column() {
    assert_eq!(
        BoardIndex::from_row_col(BOARD_SIZE, 0),
        Err(InputError::InvalidBoardIndex)
    );
    assert_eq!(
        BoardIndex::from_row_col(0, BOARD_SIZE),
        Err(InputError::InvalidBoardIndex)
    );
    assert_eq!(
        BoardIndex::from_row_col(u8::MAX, u8::MAX),
        Err(InputError::InvalidBoardIndex)
    );
}

#[test]
fn board_index_rejects_padding_bit_indices() {
    for row in 0..BOARD_SIZE {
        for col in BOARD_SIZE..ROW_STRIDE {
            let bit_index = u16::from(row) * u16::from(ROW_STRIDE) + u16::from(col);

            assert_eq!(
                BoardIndex::from_bit_index(bit_index),
                Err(InputError::InvalidBoardIndex)
            );
            assert_eq!(
                BoardIndex::try_from(bit_index),
                Err(InputError::InvalidBoardIndex)
            );
        }
    }
}

#[test]
fn board_index_rejects_bit_indices_beyond_padded_board() {
    assert_eq!(usize::from(FIRST_BIT_AFTER_PADDED_BOARD), BOARD_BITS);

    assert_eq!(
        BoardIndex::from_bit_index(FIRST_BIT_AFTER_PADDED_BOARD),
        Err(InputError::InvalidBoardIndex)
    );
    assert_eq!(
        BoardIndex::from_bit_index(u16::MAX),
        Err(InputError::InvalidBoardIndex)
    );
}

#[test]
fn all_playable_cells_round_trip_through_bit_index() {
    let mut count = 0usize;

    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let bit_index = u16::from(row) * u16::from(ROW_STRIDE) + u16::from(col);

            let Ok(index) = BoardIndex::from_row_col(row, col) else {
                panic!("playable row {row}, col {col} should be valid");
            };

            assert_eq!(index.bit_index(), bit_index);
            assert_eq!(BoardIndex::from_bit_index(bit_index), Ok(index));
            count += 1;
        }
    }

    assert_eq!(count, usize::from(BOARD_SIZE) * usize::from(BOARD_SIZE));
}

#[test]
fn const_context_apis_work_for_board_index() {
    assert_eq!(CONST_TOP_LEFT.row(), 0);
    assert_eq!(CONST_TOP_LEFT.col(), 0);
    assert_eq!(CONST_TOP_LEFT.bit_index(), 0);

    assert_eq!(CONST_TOP_RIGHT.row(), 0);
    assert_eq!(CONST_TOP_RIGHT.col(), 19);
    assert_eq!(CONST_TOP_RIGHT.bit_index(), 19);

    assert_eq!(CONST_BOTTOM_RIGHT.row(), 19);
    assert_eq!(CONST_BOTTOM_RIGHT.col(), 19);
    assert_eq!(CONST_BOTTOM_RIGHT.bit_index(), 627);

    assert_eq!(CONST_TOP_LEFT_FROM_BIT, CONST_TOP_LEFT);

    assert_eq!(CONST_TOP_LEFT_BIT_INDEX, 0);
    assert_eq!(CONST_TOP_LEFT_LANE, 0);

    assert_eq!(CONST_TOP_LEFT.row(), 0);

    assert_eq!(CONST_TOP_LEFT.col(), 0);

    assert_eq!(CONST_TOP_LEFT.offset(), 0);

    assert_eq!(CONST_TOP_LEFT.lane_bit(), 1);
}

#[test]
fn board_index_is_copy_comparable_ordered_and_hashable() {
    let Ok(first) = BoardIndex::from_row_col(0, 0) else {
        panic!("top-left should be valid");
    };

    let Ok(duplicate) = BoardIndex::from_bit_index(0) else {
        panic!("bit index 0 should be valid");
    };

    let Ok(second) = BoardIndex::from_row_col(0, 1) else {
        panic!("top row second column should be valid");
    };

    let copied = first;

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(first < second);

    let mut indices = HashSet::new();
    indices.insert(first);
    indices.insert(duplicate);
    indices.insert(second);

    assert_eq!(indices.len(), 2);
    assert!(indices.contains(&first));
    assert!(indices.contains(&second));
}

#[test]
fn board_index_supports_try_from_u16_and_into_u16() {
    let Ok(index) = BoardIndex::try_from(0u16) else {
        panic!("bit index 0 should be valid");
    };

    let raw: u16 = index.into();

    assert_eq!(raw, 0);
    assert_eq!(
        BoardIndex::try_from(20u16),
        Err(InputError::InvalidBoardIndex)
    );
}

#[test]
fn board_index_displays_padded_bit_index() {
    let Ok(index) = BoardIndex::from_row_col(1, 2) else {
        panic!("row 1 col 2 should be valid");
    };

    assert_eq!(index.bit_index(), 34);
    assert_eq!(index.to_string(), "34");
}

#[test]
fn board_index_runtime_accessors_cover_row_col_lane_offset_and_lane_bit() {
    let Ok(index) = BoardIndex::from_row_col(4, 3) else {
        panic!("row 4 col 3 should be valid");
    };

    assert_eq!(index.row(), 4);
    assert_eq!(index.col(), 3);
    assert_eq!(index.bit_index(), 131);
    assert_eq!(index.lane(), 1);
    assert_eq!(index.offset(), 3);
    assert_eq!(index.lane_bit(), 8);
}
