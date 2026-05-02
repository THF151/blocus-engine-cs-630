use blocus_core::{
    DomainEventKind, DomainResponseKind, LastPieceByColor, PIECE_COUNT, PieceId, PlayerColor,
};

fn piece_id(value: u8) -> PieceId {
    PieceId::try_new(value).unwrap_or_else(|_| panic!("piece id {value} should be valid"))
}

#[test]
fn domain_event_kind_as_str_covers_all_variants() {
    assert_eq!(DomainEventKind::MoveApplied.as_str(), "move_applied");
    assert_eq!(DomainEventKind::PlayerPassed.as_str(), "player_passed");
    assert_eq!(DomainEventKind::TurnAdvanced.as_str(), "turn_advanced");
    assert_eq!(DomainEventKind::GameFinished.as_str(), "game_finished");
}

#[test]
fn domain_response_kind_as_str_covers_all_variants() {
    assert_eq!(DomainResponseKind::MoveApplied.as_str(), "move_applied");
    assert_eq!(DomainResponseKind::PlayerPassed.as_str(), "player_passed");
    assert_eq!(DomainResponseKind::GameFinished.as_str(), "game_finished");
}

#[test]
fn last_piece_by_color_from_packed_and_packed_round_trip_valid_slots() {
    let blue_encoded = u32::from(piece_id(0).as_u8()) + 1;
    let yellow_encoded = u32::from(piece_id(20).as_u8()) + 1;
    let red_encoded = u32::from(piece_id(7).as_u8()) + 1;
    let green_encoded = u32::from(piece_id(12).as_u8()) + 1;

    let packed = blue_encoded | (yellow_encoded << 5) | (red_encoded << 10) | (green_encoded << 15);

    let tracker = LastPieceByColor::from_packed(packed);

    assert_eq!(tracker.packed(), packed);
    assert_eq!(tracker.get(PlayerColor::Blue), Some(piece_id(0)));
    assert_eq!(tracker.get(PlayerColor::Yellow), Some(piece_id(20)));
    assert_eq!(tracker.get(PlayerColor::Red), Some(piece_id(7)));
    assert_eq!(tracker.get(PlayerColor::Green), Some(piece_id(12)));
}

#[test]
fn last_piece_by_color_from_packed_masks_bits_outside_four_slots() {
    let valid_packed = 1;
    let noisy_packed = valid_packed | (u32::MAX << 20);

    let tracker = LastPieceByColor::from_packed(noisy_packed);

    assert_eq!(tracker.packed(), valid_packed);
    assert_eq!(tracker.get(PlayerColor::Blue), Some(piece_id(0)));
    assert_eq!(tracker.get(PlayerColor::Yellow), None);
    assert_eq!(tracker.get(PlayerColor::Red), None);
    assert_eq!(tracker.get(PlayerColor::Green), None);
}
#[test]
fn last_piece_by_color_get_returns_none_for_empty_and_invalid_encoded_slots() {
    let empty_tracker = LastPieceByColor::from_packed(0);

    assert_eq!(empty_tracker.get(PlayerColor::Blue), None);
    assert_eq!(empty_tracker.get(PlayerColor::Yellow), None);
    assert_eq!(empty_tracker.get(PlayerColor::Red), None);
    assert_eq!(empty_tracker.get(PlayerColor::Green), None);

    let invalid_piece_slot = u32::from(PIECE_COUNT) + 1;
    let invalid_tracker = LastPieceByColor::from_packed(invalid_piece_slot);

    assert_eq!(invalid_tracker.get(PlayerColor::Blue), None);
}

#[test]
fn last_piece_by_color_with_set_records_piece_without_mutating_original() {
    let original = LastPieceByColor::EMPTY;
    let updated = original.with_set(PlayerColor::Green, piece_id(20));

    assert_eq!(original.get(PlayerColor::Green), None);
    assert_eq!(updated.get(PlayerColor::Green), Some(piece_id(20)));
    assert_ne!(original.packed(), updated.packed());
}
