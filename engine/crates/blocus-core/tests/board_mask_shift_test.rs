use blocus_core::{BOARD_SIZE, BoardIndex, BoardMask, PLAYABLE_MASK};

fn index(row: u8, col: u8) -> BoardIndex {
    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|_| panic!("row {row}, col {col} should be playable"))
}

fn singleton(row: u8, col: u8) -> BoardMask {
    BoardMask::from_index(index(row, col))
}

#[test]
fn shift_north_moves_interior_cell_one_row_up() {
    assert_eq!(singleton(10, 7).shift_north(), singleton(9, 7));
}

#[test]
fn shift_south_moves_interior_cell_one_row_down() {
    assert_eq!(singleton(10, 7).shift_south(), singleton(11, 7));
}

#[test]
fn shift_east_moves_interior_cell_one_column_right() {
    assert_eq!(singleton(10, 7).shift_east(), singleton(10, 8));
}

#[test]
fn shift_west_moves_interior_cell_one_column_left() {
    assert_eq!(singleton(10, 7).shift_west(), singleton(10, 6));
}

#[test]
fn shift_north_from_top_row_drops_cell() {
    assert_eq!(singleton(0, 7).shift_north(), BoardMask::EMPTY);
}

#[test]
fn shift_south_from_bottom_row_drops_cell() {
    assert_eq!(singleton(BOARD_SIZE - 1, 7).shift_south(), BoardMask::EMPTY);
}

#[test]
fn shift_east_from_right_edge_drops_cell_instead_of_creating_padding_bit() {
    assert_eq!(singleton(7, BOARD_SIZE - 1).shift_east(), BoardMask::EMPTY);
}

#[test]
fn shift_west_from_left_edge_drops_cell_instead_of_wrapping_to_padding_bit() {
    assert_eq!(singleton(7, 0).shift_west(), BoardMask::EMPTY);
}

#[test]
fn shift_east_from_column_eighteen_reaches_right_edge() {
    assert_eq!(
        singleton(7, BOARD_SIZE - 2).shift_east(),
        singleton(7, BOARD_SIZE - 1)
    );
}

#[test]
fn shift_west_from_column_one_reaches_left_edge() {
    assert_eq!(singleton(7, 1).shift_west(), singleton(7, 0));
}

#[test]
fn shift_north_across_lane_boundary_moves_row_four_to_row_three() {
    assert_eq!(singleton(4, 5).shift_north(), singleton(3, 5));
}

#[test]
fn shift_south_across_lane_boundary_moves_row_three_to_row_four() {
    assert_eq!(singleton(3, 5).shift_south(), singleton(4, 5));
}

#[test]
fn directional_shifts_preserve_playable_subset_for_all_playable_singletons() {
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let mask = singleton(row, col);

            assert!(mask.shift_north().is_playable_subset());
            assert!(mask.shift_south().is_playable_subset());
            assert!(mask.shift_east().is_playable_subset());
            assert!(mask.shift_west().is_playable_subset());
        }
    }
}

#[test]
fn east_shift_drops_every_right_edge_cell() {
    for row in 0..BOARD_SIZE {
        assert_eq!(
            singleton(row, BOARD_SIZE - 1).shift_east(),
            BoardMask::EMPTY
        );
    }
}

#[test]
fn west_shift_drops_every_left_edge_cell() {
    for row in 0..BOARD_SIZE {
        assert_eq!(singleton(row, 0).shift_west(), BoardMask::EMPTY);
    }
}

#[test]
fn raw_padding_bit_does_not_intersect_playable_mask() {
    let padding_bit = BoardMask::from_lanes([1u128 << BOARD_SIZE, 0, 0, 0, 0]);

    assert!(!padding_bit.is_playable_subset());
    assert!(!padding_bit.intersects(PLAYABLE_MASK));
}

#[test]
fn shift_east_does_not_create_row_padding_bit() {
    let shifted = singleton(0, BOARD_SIZE - 1).shift_east();

    assert_eq!(shifted, BoardMask::EMPTY);
    assert!(shifted.is_playable_subset());
}

#[test]
fn shift_west_does_not_wrap_into_previous_row_padding_bit() {
    let shifted = singleton(1, 0).shift_west();

    assert_eq!(shifted, BoardMask::EMPTY);
    assert!(shifted.is_playable_subset());
}

#[test]
fn diagonal_shift_from_top_left_corner_drops_out_of_bounds_components() {
    let mask = singleton(0, 0);

    assert_eq!(mask.shift_north().shift_west(), BoardMask::EMPTY);
    assert_eq!(mask.shift_north().shift_east(), BoardMask::EMPTY);
    assert_eq!(mask.shift_south().shift_west(), BoardMask::EMPTY);
    assert_eq!(mask.shift_south().shift_east(), singleton(1, 1));
}

#[test]
fn diagonal_shift_from_bottom_right_corner_drops_out_of_bounds_components() {
    let mask = singleton(BOARD_SIZE - 1, BOARD_SIZE - 1);

    assert_eq!(mask.shift_south().shift_east(), BoardMask::EMPTY);
    assert_eq!(mask.shift_south().shift_west(), BoardMask::EMPTY);
    assert_eq!(mask.shift_north().shift_east(), BoardMask::EMPTY);
    assert_eq!(
        mask.shift_north().shift_west(),
        singleton(BOARD_SIZE - 2, BOARD_SIZE - 2)
    );
}

#[test]
fn intersection_keeps_only_shared_bits() {
    let first = singleton(3, 4).union(singleton(5, 6));
    let second = singleton(5, 6).union(singleton(7, 8));

    assert_eq!(first.intersection(second), singleton(5, 6));
}

#[test]
fn intersection_with_empty_is_empty() {
    assert_eq!(
        singleton(3, 4).intersection(BoardMask::EMPTY),
        BoardMask::EMPTY
    );
    assert_eq!(
        BoardMask::EMPTY.intersection(singleton(3, 4)),
        BoardMask::EMPTY
    );
}

#[test]
fn intersection_with_self_preserves_mask() {
    let mask = singleton(3, 4).union(singleton(5, 6));

    assert_eq!(mask.intersection(mask), mask);
}
