use blocus_core::{BoardIndex, BoardMask, BoardState, PLAYER_COLOR_COUNT, PlayerColor};
use std::collections::HashSet;

fn index(row: u8, col: u8) -> BoardIndex {
    let Ok(index) = BoardIndex::from_row_col(row, col) else {
        panic!("row {row}, col {col} should be valid");
    };

    index
}

#[test]
fn empty_board_state_has_no_occupancy() {
    let state = BoardState::EMPTY;

    assert_eq!(
        state.occupied_by_color(),
        [BoardMask::EMPTY; PLAYER_COLOR_COUNT]
    );
    assert!(state.is_empty());
    assert_eq!(state.occupied_all(), BoardMask::EMPTY);

    for color in PlayerColor::ALL {
        assert_eq!(state.occupied(color), BoardMask::EMPTY);
    }
}

#[test]
fn default_board_state_is_empty() {
    assert_eq!(BoardState::default(), BoardState::EMPTY);
}

#[test]
fn from_occupied_by_color_preserves_per_color_masks() {
    let blue = BoardMask::from_index(index(0, 0));
    let yellow = BoardMask::from_index(index(0, 19));
    let red = BoardMask::from_index(index(19, 19));
    let green = BoardMask::from_index(index(19, 0));

    let masks = [blue, yellow, red, green];
    let state = BoardState::from_occupied_by_color(masks);

    assert_eq!(state.occupied_by_color(), masks);
    assert_eq!(state.occupied(PlayerColor::Blue), blue);
    assert_eq!(state.occupied(PlayerColor::Yellow), yellow);
    assert_eq!(state.occupied(PlayerColor::Red), red);
    assert_eq!(state.occupied(PlayerColor::Green), green);
}

#[test]
fn occupied_mut_allows_targeted_color_update() {
    let mut state = BoardState::EMPTY;
    let blue_mask = BoardMask::from_index(index(1, 1));

    *state.occupied_mut(PlayerColor::Blue) = blue_mask;

    assert_eq!(state.occupied(PlayerColor::Blue), blue_mask);
    assert_eq!(state.occupied(PlayerColor::Yellow), BoardMask::EMPTY);
    assert_eq!(state.occupied(PlayerColor::Red), BoardMask::EMPTY);
    assert_eq!(state.occupied(PlayerColor::Green), BoardMask::EMPTY);
}

#[test]
fn place_mask_unions_new_cells_with_existing_color_occupancy() {
    let mut state = BoardState::EMPTY;

    let first = BoardMask::from_index(index(0, 0));
    let second = BoardMask::from_index(index(1, 1));

    state.place_mask(PlayerColor::Blue, first);
    state.place_mask(PlayerColor::Blue, second);

    let occupied = state.occupied(PlayerColor::Blue);

    assert_eq!(occupied.count(), 2);
    assert!(occupied.contains(index(0, 0)));
    assert!(occupied.contains(index(1, 1)));
}

#[test]
fn place_mask_updates_only_selected_color() {
    let mut state = BoardState::EMPTY;

    let blue = BoardMask::from_index(index(0, 0));
    let yellow = BoardMask::from_index(index(0, 19));

    state.place_mask(PlayerColor::Blue, blue);
    state.place_mask(PlayerColor::Yellow, yellow);

    assert_eq!(state.occupied(PlayerColor::Blue), blue);
    assert_eq!(state.occupied(PlayerColor::Yellow), yellow);
    assert_eq!(state.occupied(PlayerColor::Red), BoardMask::EMPTY);
    assert_eq!(state.occupied(PlayerColor::Green), BoardMask::EMPTY);
}

#[test]
fn occupied_all_derives_union_of_all_color_masks() {
    let blue = BoardMask::from_index(index(0, 0));
    let yellow = BoardMask::from_index(index(0, 19));
    let red = BoardMask::from_index(index(19, 19));
    let green = BoardMask::from_index(index(19, 0));

    let state = BoardState::from_occupied_by_color([blue, yellow, red, green]);
    let occupied_all = state.occupied_all();

    assert_eq!(occupied_all.count(), 4);
    assert!(occupied_all.contains(index(0, 0)));
    assert!(occupied_all.contains(index(0, 19)));
    assert!(occupied_all.contains(index(19, 19)));
    assert!(occupied_all.contains(index(19, 0)));
}

#[test]
fn is_empty_returns_false_when_any_color_is_occupied() {
    let mut state = BoardState::EMPTY;

    assert!(state.is_empty());

    state.place_mask(PlayerColor::Green, BoardMask::from_index(index(19, 0)));

    assert!(!state.is_empty());
}

#[test]
fn board_state_is_copy_eq_hash_and_debug() {
    let mut state = BoardState::EMPTY;
    state.place_mask(PlayerColor::Blue, BoardMask::from_index(index(0, 0)));

    let duplicate = state;
    let copied = state;

    let mut other = BoardState::EMPTY;
    other.place_mask(PlayerColor::Yellow, BoardMask::from_index(index(0, 19)));

    assert_eq!(state, copied);
    assert_eq!(state, duplicate);
    assert_ne!(state, other);
    assert!(format!("{state:?}").contains("BoardState"));

    let mut states = HashSet::new();
    states.insert(state);
    states.insert(duplicate);
    states.insert(other);

    assert_eq!(states.len(), 2);
}
