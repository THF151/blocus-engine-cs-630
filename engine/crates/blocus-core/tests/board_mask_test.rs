use blocus_core::{
    BOARD_BITS, BOARD_LANES, BOARD_SIZE, BoardIndex, BoardMask, InputError, PLAYABLE_CELLS,
    PLAYABLE_MASK, ROW_PADDING_BITS, ROW_STRIDE,
};
use std::collections::HashSet;

const FIRST_LANE_INDEX: BoardIndex = match BoardIndex::from_bit_index(0) {
    Ok(index) => index,
    Err(_) => panic!("bit index 0 should be playable"),
};

const SECOND_LANE_INDEX: BoardIndex = match BoardIndex::from_bit_index(128) {
    Ok(index) => index,
    Err(_) => panic!("bit index 128 should be playable"),
};

const THIRD_LANE_INDEX: BoardIndex = match BoardIndex::from_bit_index(256) {
    Ok(index) => index,
    Err(_) => panic!("bit index 256 should be playable"),
};

const FOURTH_LANE_INDEX: BoardIndex = match BoardIndex::from_bit_index(384) {
    Ok(index) => index,
    Err(_) => panic!("bit index 384 should be playable"),
};

const FIFTH_LANE_INDEX: BoardIndex = match BoardIndex::from_bit_index(512) {
    Ok(index) => index,
    Err(_) => panic!("bit index 512 should be playable"),
};

const LANE_REPRESENTATIVES: [BoardIndex; BOARD_LANES] = [
    FIRST_LANE_INDEX,
    SECOND_LANE_INDEX,
    THIRD_LANE_INDEX,
    FOURTH_LANE_INDEX,
    FIFTH_LANE_INDEX,
];

#[test]
fn empty_mask_has_no_lanes_set() {
    let mask = BoardMask::EMPTY;

    assert_eq!(mask.lanes(), [0; BOARD_LANES]);
    assert_eq!(mask.count(), 0);
    assert!(mask.is_empty());
    assert!(mask.is_playable_subset());
}

#[test]
fn default_mask_is_empty() {
    assert_eq!(BoardMask::default(), BoardMask::EMPTY);
}

#[test]
fn from_lanes_preserves_raw_lane_layout() {
    let lanes = [1, 2, 4, 8, 16];
    let mask = BoardMask::from_lanes(lanes);

    assert_eq!(mask.lanes(), lanes);
    assert_eq!(mask.count(), 5);
}

#[test]
fn from_index_sets_exactly_one_bit_in_each_representative_lane() {
    for index in LANE_REPRESENTATIVES {
        let mask = BoardMask::from_index(index);
        let lanes = mask.lanes();

        assert_eq!(mask.count(), 1);
        assert!(mask.contains(index));

        for (lane, lane_value) in lanes.iter().enumerate() {
            if lane == index.lane() {
                assert_eq!(*lane_value, index.lane_bit());
            } else {
                assert_eq!(*lane_value, 0);
            }
        }
    }
}

#[test]
fn inserted_returns_new_mask_without_mutating_original() {
    let original = BoardMask::EMPTY;
    let inserted = original.inserted(FIRST_LANE_INDEX);

    assert!(original.is_empty());
    assert!(!inserted.is_empty());
    assert!(inserted.contains(FIRST_LANE_INDEX));
}

#[test]
fn insert_mutates_mask_in_place() {
    let mut mask = BoardMask::EMPTY;

    mask.insert(FIRST_LANE_INDEX);
    mask.insert(SECOND_LANE_INDEX);

    assert_eq!(mask.count(), 2);
    assert!(mask.contains(FIRST_LANE_INDEX));
    assert!(mask.contains(SECOND_LANE_INDEX));
}

#[test]
fn contains_returns_false_for_missing_index() {
    let mask = BoardMask::from_index(FIRST_LANE_INDEX);

    assert!(mask.contains(FIRST_LANE_INDEX));
    assert!(!mask.contains(SECOND_LANE_INDEX));
}

#[test]
fn intersects_detects_shared_bits() {
    let first = BoardMask::from_index(FIRST_LANE_INDEX);
    let same = BoardMask::from_index(FIRST_LANE_INDEX);
    let different = BoardMask::from_index(SECOND_LANE_INDEX);

    assert!(first.intersects(same));
    assert!(!first.intersects(different));
    assert!(!first.intersects(BoardMask::EMPTY));
}

#[test]
fn union_combines_bits_across_lanes() {
    let first = BoardMask::from_index(FIRST_LANE_INDEX);
    let second = BoardMask::from_index(SECOND_LANE_INDEX);

    let union = first.union(second);

    assert_eq!(union.count(), 2);
    assert!(union.contains(FIRST_LANE_INDEX));
    assert!(union.contains(SECOND_LANE_INDEX));
}

#[test]
fn difference_removes_bits_present_in_other_mask() {
    let first = BoardMask::from_index(FIRST_LANE_INDEX);
    let second = BoardMask::from_index(SECOND_LANE_INDEX);
    let combined = first.union(second);

    let difference = combined.difference(first);

    assert_eq!(difference, second);
    assert!(!difference.contains(FIRST_LANE_INDEX));
    assert!(difference.contains(SECOND_LANE_INDEX));
}

#[test]
fn is_subset_of_recognizes_subset_relationships() {
    let first = BoardMask::from_index(FIRST_LANE_INDEX);
    let second = BoardMask::from_index(SECOND_LANE_INDEX);
    let combined = first.union(second);

    assert!(BoardMask::EMPTY.is_subset_of(first));
    assert!(first.is_subset_of(first));
    assert!(first.is_subset_of(combined));
    assert!(!combined.is_subset_of(first));
}

#[test]
fn count_counts_bits_across_every_lane() {
    let mut mask = BoardMask::EMPTY;

    for index in LANE_REPRESENTATIVES {
        mask.insert(index);
    }

    assert_eq!(mask.count(), 5);
}

#[test]
fn mask_operations_preserve_copy_equality_hash_and_debug() {
    let first = BoardMask::from_index(FIRST_LANE_INDEX);
    let duplicate = BoardMask::from_index(FIRST_LANE_INDEX);
    let second = BoardMask::from_index(SECOND_LANE_INDEX);
    let copied = first;

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(format!("{first:?}").contains("BoardMask"));

    let mut masks = HashSet::new();
    masks.insert(first);
    masks.insert(duplicate);
    masks.insert(second);

    assert_eq!(masks.len(), 2);
    assert!(masks.contains(&BoardMask::from_index(FIRST_LANE_INDEX)));
    assert!(masks.contains(&BoardMask::from_index(SECOND_LANE_INDEX)));
}

#[test]
fn playable_mask_contains_every_valid_row_col_cell() {
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let Ok(index) = BoardIndex::from_row_col(row, col) else {
                panic!("row {row}, col {col} should be playable");
            };

            assert!(PLAYABLE_MASK.contains(index));
        }
    }
}

#[test]
fn playable_mask_excludes_every_row_padding_bit() {
    for row in 0..BOARD_SIZE {
        for col in BOARD_SIZE..ROW_STRIDE {
            let bit_index = u16::from(row) * u16::from(ROW_STRIDE) + u16::from(col);

            assert_eq!(
                BoardIndex::from_bit_index(bit_index),
                Err(InputError::InvalidBoardIndex)
            );

            let lane = usize::from(bit_index) / 128;
            let offset = usize::from(bit_index) % 128;
            let padding_mask = BoardMask::from_lanes({
                let mut lanes = [0u128; BOARD_LANES];
                lanes[lane] = 1u128 << offset;
                lanes
            });

            assert!(!padding_mask.is_playable_subset());
            assert!(!PLAYABLE_MASK.intersects(padding_mask));
        }
    }
}

#[test]
fn playable_mask_has_exactly_four_hundred_cells() {
    let playable_cells = u32::try_from(PLAYABLE_CELLS)
        .unwrap_or_else(|_| panic!("PLAYABLE_CELLS should fit in u32"));

    assert_eq!(PLAYABLE_MASK.count(), playable_cells);
}

#[test]
fn board_geometry_accounts_for_playable_and_padding_bits() {
    assert_eq!(BOARD_BITS, 640);
    assert_eq!(BOARD_LANES, 5);
    assert_eq!(PLAYABLE_CELLS, 400);
    assert_eq!(ROW_PADDING_BITS, 12);
    let padded_cells = PLAYABLE_CELLS + usize::from(BOARD_SIZE) * usize::from(ROW_PADDING_BITS);
    assert_eq!(padded_cells, BOARD_BITS);
}

#[test]
fn playable_subset_accepts_only_playable_bits() {
    let playable = BoardMask::from_index(FIRST_LANE_INDEX);
    let non_playable = BoardMask::from_lanes([1u128 << 20, 0, 0, 0, 0]);

    assert!(playable.is_playable_subset());
    assert!(!non_playable.is_playable_subset());
}

#[test]
fn intersects_checks_all_lanes_until_last_lane_match() {
    let last_lane_index = FIFTH_LANE_INDEX;

    let first = BoardMask::from_index(last_lane_index);
    let second = BoardMask::from_index(last_lane_index);

    assert!(first.intersects(second));
}

#[test]
fn intersects_returns_false_after_checking_all_lanes() {
    let first = BoardMask::from_index(FIRST_LANE_INDEX);
    let second = BoardMask::from_index(FIFTH_LANE_INDEX);

    assert!(!first.difference(first).intersects(second));
}

#[test]
fn union_with_empty_preserves_mask() {
    let mask = BoardMask::from_index(THIRD_LANE_INDEX);

    assert_eq!(mask.union(BoardMask::EMPTY), mask);
    assert_eq!(BoardMask::EMPTY.union(mask), mask);
}

#[test]
fn difference_with_empty_preserves_mask() {
    let mask = BoardMask::from_index(FOURTH_LANE_INDEX);

    assert_eq!(mask.difference(BoardMask::EMPTY), mask);
}

#[test]
fn difference_with_self_returns_empty() {
    let mask = BoardMask::from_index(FOURTH_LANE_INDEX);

    assert_eq!(mask.difference(mask), BoardMask::EMPTY);
    assert!(mask.difference(mask).is_empty());
}

#[test]
fn is_empty_checks_non_empty_masks() {
    let mask = BoardMask::from_index(FIFTH_LANE_INDEX);

    assert!(!mask.is_empty());
}

#[test]
fn is_subset_of_checks_all_lanes_for_true_case() {
    let mut subset = BoardMask::EMPTY;
    subset.insert(FIRST_LANE_INDEX);
    subset.insert(FIFTH_LANE_INDEX);

    let superset = subset
        .inserted(SECOND_LANE_INDEX)
        .inserted(THIRD_LANE_INDEX)
        .inserted(FOURTH_LANE_INDEX);

    assert!(subset.is_subset_of(superset));
}

#[test]
fn is_subset_of_returns_false_for_missing_last_lane_bit() {
    let first_lane = BoardMask::from_index(FIRST_LANE_INDEX);
    let last_lane = BoardMask::from_index(FIFTH_LANE_INDEX);
    let combined = first_lane.union(last_lane);

    assert!(!combined.is_subset_of(first_lane));
}

#[test]
fn count_returns_zero_for_empty_mask_at_runtime() {
    assert_eq!(BoardMask::EMPTY.count(), 0);
}

#[test]
fn count_counts_multiple_bits_in_same_lane() {
    let Ok(first) = BoardIndex::from_row_col(0, 0) else {
        panic!("row 0 col 0 should be valid");
    };
    let Ok(second) = BoardIndex::from_row_col(0, 1) else {
        panic!("row 0 col 1 should be valid");
    };
    let Ok(third) = BoardIndex::from_row_col(0, 2) else {
        panic!("row 0 col 2 should be valid");
    };

    let mask = BoardMask::EMPTY
        .inserted(first)
        .inserted(second)
        .inserted(third);

    assert_eq!(mask.count(), 3);
}

#[test]
fn lanes_accessor_returns_copy_not_mutable_alias() {
    let mask = BoardMask::from_index(FIRST_LANE_INDEX);
    let mut lanes = mask.lanes();

    lanes[0] = 0;

    assert!(mask.contains(FIRST_LANE_INDEX));
    assert_eq!(lanes[0], 0);
}
