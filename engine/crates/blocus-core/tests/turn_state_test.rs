use blocus_core::{PlayerColor, PlayerId, PlayerSlots, SharedColorTurn, TurnOrder, TurnState};
use std::collections::HashSet;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn player_id(value: u128) -> PlayerId {
    PlayerId::from_uuid(uuid(value))
}

fn custom_turn_order() -> TurnOrder {
    let Ok(order) = TurnOrder::try_new([
        PlayerColor::Red,
        PlayerColor::Green,
        PlayerColor::Blue,
        PlayerColor::Yellow,
    ]) else {
        panic!("custom turn order should be valid");
    };

    order
}

fn two_player_slots() -> PlayerSlots {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    slots
}

fn three_player_slots() -> PlayerSlots {
    let Ok(shared) = SharedColorTurn::try_new(
        PlayerColor::Green,
        [player_id(1), player_id(2), player_id(3)],
    ) else {
        panic!("shared color turn should be valid");
    };

    let Ok(slots) = PlayerSlots::three_player(
        [
            (PlayerColor::Blue, player_id(1)),
            (PlayerColor::Yellow, player_id(2)),
            (PlayerColor::Red, player_id(3)),
        ],
        shared,
    ) else {
        panic!("three-player slots should be valid");
    };

    slots
}

#[test]
fn turn_state_starts_at_first_color_of_turn_order() {
    let official = TurnState::new(TurnOrder::OFFICIAL_FIXED);
    let custom = TurnState::new(custom_turn_order());

    assert_eq!(official.current_color(), PlayerColor::Blue);
    assert_eq!(custom.current_color(), PlayerColor::Red);

    assert_eq!(official.passed_mask(), 0);
    assert_eq!(official.shared_color_turn_index(), 0);
    assert_eq!(official.passed_count(), 0);
    assert!(!official.all_colors_passed());
}

#[test]
fn from_parts_masks_unknown_passed_bits() {
    let turn = TurnState::from_parts(PlayerColor::Yellow, u8::MAX, 4);

    assert_eq!(turn.current_color(), PlayerColor::Yellow);
    assert_eq!(turn.passed_mask(), 0b11_1111);
    assert_eq!(turn.shared_color_turn_index(), 4);
    assert_eq!(turn.passed_count(), 6);
    assert!(turn.all_colors_passed());
}

#[test]
fn mark_passed_sets_color_bit_and_is_idempotent() {
    let mut turn = TurnState::new(TurnOrder::OFFICIAL_FIXED);

    turn.mark_passed(PlayerColor::Yellow);

    assert!(turn.is_passed(PlayerColor::Yellow));
    assert!(!turn.is_passed(PlayerColor::Blue));
    assert_eq!(turn.passed_count(), 1);

    turn.mark_passed(PlayerColor::Yellow);

    assert_eq!(turn.passed_count(), 1);
}

#[test]
fn marked_passed_returns_updated_copy() {
    let original = TurnState::new(TurnOrder::OFFICIAL_FIXED);
    let updated = original.marked_passed(PlayerColor::Blue);

    assert!(!original.is_passed(PlayerColor::Blue));
    assert!(updated.is_passed(PlayerColor::Blue));
}

#[test]
fn current_player_uses_player_slots() {
    let slots = two_player_slots();
    let turn = TurnState::new(TurnOrder::OFFICIAL_FIXED);

    assert_eq!(turn.current_color(), PlayerColor::Blue);
    assert_eq!(turn.current_player(slots), Some(player_id(1)));
}

#[test]
fn advance_uses_official_order_for_two_player_variant() {
    let slots = two_player_slots();
    let mut turn = TurnState::new(TurnOrder::OFFICIAL_FIXED);

    assert_eq!(turn.advance(TurnOrder::OFFICIAL_FIXED, slots), Some(turn));
    assert_eq!(turn.current_color(), PlayerColor::Yellow);

    assert_eq!(turn.advance(TurnOrder::OFFICIAL_FIXED, slots), Some(turn));
    assert_eq!(turn.current_color(), PlayerColor::Red);

    assert_eq!(turn.advance(TurnOrder::OFFICIAL_FIXED, slots), Some(turn));
    assert_eq!(turn.current_color(), PlayerColor::Green);

    assert_eq!(turn.advance(TurnOrder::OFFICIAL_FIXED, slots), Some(turn));
    assert_eq!(turn.current_color(), PlayerColor::Blue);
}

#[test]
fn advance_uses_custom_order_for_four_player_variant() {
    let Ok(slots) = PlayerSlots::four_player([
        (PlayerColor::Blue, player_id(1)),
        (PlayerColor::Yellow, player_id(2)),
        (PlayerColor::Red, player_id(3)),
        (PlayerColor::Green, player_id(4)),
    ]) else {
        panic!("four-player slots should be valid");
    };

    let order = custom_turn_order();
    let mut turn = TurnState::new(order);

    assert_eq!(turn.current_color(), PlayerColor::Red);

    turn.advance(order, slots);
    assert_eq!(turn.current_color(), PlayerColor::Green);

    turn.advance(order, slots);
    assert_eq!(turn.current_color(), PlayerColor::Blue);

    turn.advance(order, slots);
    assert_eq!(turn.current_color(), PlayerColor::Yellow);

    turn.advance(order, slots);
    assert_eq!(turn.current_color(), PlayerColor::Red);
}

#[test]
fn advance_skips_passed_colors() {
    let slots = two_player_slots();
    let mut turn = TurnState::new(TurnOrder::OFFICIAL_FIXED)
        .marked_passed(PlayerColor::Yellow)
        .marked_passed(PlayerColor::Red);

    assert_eq!(turn.current_color(), PlayerColor::Blue);

    assert_eq!(turn.advance(TurnOrder::OFFICIAL_FIXED, slots), Some(turn));
    assert_eq!(turn.current_color(), PlayerColor::Green);

    assert_eq!(turn.advance(TurnOrder::OFFICIAL_FIXED, slots), Some(turn));
    assert_eq!(turn.current_color(), PlayerColor::Blue);
}

#[test]
fn advance_returns_none_when_all_colors_have_passed() {
    let slots = two_player_slots();
    let mut turn = TurnState::new(TurnOrder::OFFICIAL_FIXED)
        .marked_passed(PlayerColor::Blue)
        .marked_passed(PlayerColor::Yellow)
        .marked_passed(PlayerColor::Red)
        .marked_passed(PlayerColor::Green);

    assert!(turn.all_colors_passed());
    assert_eq!(turn.advance(TurnOrder::OFFICIAL_FIXED, slots), None);
}

#[test]
fn shared_color_alternates_when_shared_color_turn_is_advanced() {
    let slots = three_player_slots();

    let mut turn = TurnState::from_parts(PlayerColor::Green, 0, 0);

    assert_eq!(turn.current_player(slots), Some(player_id(1)));

    turn.advance(TurnOrder::OFFICIAL_FIXED, slots);
    assert_eq!(turn.shared_color_turn_index(), 1);

    turn = TurnState::from_parts(
        PlayerColor::Green,
        turn.passed_mask(),
        turn.shared_color_turn_index(),
    );
    assert_eq!(turn.current_player(slots), Some(player_id(2)));

    turn.advance(TurnOrder::OFFICIAL_FIXED, slots);
    assert_eq!(turn.shared_color_turn_index(), 2);

    turn = TurnState::from_parts(
        PlayerColor::Green,
        turn.passed_mask(),
        turn.shared_color_turn_index(),
    );
    assert_eq!(turn.current_player(slots), Some(player_id(3)));
}

#[test]
fn non_shared_color_does_not_advance_shared_color_index() {
    let slots = three_player_slots();

    let mut turn = TurnState::from_parts(PlayerColor::Blue, 0, 0);

    turn.advance(TurnOrder::OFFICIAL_FIXED, slots);

    assert_eq!(turn.shared_color_turn_index(), 0);
}

#[test]
fn turn_state_is_copy_eq_hash_and_debug() {
    let first = TurnState::new(TurnOrder::OFFICIAL_FIXED);
    let duplicate = first;
    let copied = first;
    let second = first.marked_passed(PlayerColor::Blue);

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(format!("{first:?}").contains("TurnState"));

    let mut turns = HashSet::new();
    turns.insert(first);
    turns.insert(duplicate);
    turns.insert(second);

    assert_eq!(turns.len(), 2);
}
