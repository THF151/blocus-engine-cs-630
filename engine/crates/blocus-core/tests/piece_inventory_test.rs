use blocus_core::pieces::ALL_PIECES_MASK;
use blocus_core::{PIECE_COUNT, PieceId, PieceInventory};
use std::collections::HashSet;

fn piece_id(value: u8) -> PieceId {
    let Ok(piece) = PieceId::try_new(value) else {
        panic!("piece id {value} should be valid");
    };

    piece
}

#[test]
fn all_pieces_mask_contains_exactly_the_official_piece_bits() {
    assert_eq!(PIECE_COUNT, 21);
    assert_eq!(ALL_PIECES_MASK, (1u32 << 21) - 1);
    assert_eq!(ALL_PIECES_MASK.count_ones(), u32::from(PIECE_COUNT));
}

#[test]
fn empty_inventory_has_all_pieces_available() {
    let inventory = PieceInventory::EMPTY;

    assert_eq!(inventory.used_mask(), 0);
    assert_eq!(inventory.available_mask(), ALL_PIECES_MASK);
    assert_eq!(inventory.used_count(), 0);
    assert_eq!(inventory.available_count(), u32::from(PIECE_COUNT));
    assert!(!inventory.is_complete());

    for value in 0..PIECE_COUNT {
        let piece = piece_id(value);

        assert!(!inventory.is_used(piece));
        assert!(inventory.is_available(piece));
    }
}

#[test]
fn default_inventory_is_empty() {
    assert_eq!(PieceInventory::default(), PieceInventory::EMPTY);
}

#[test]
fn from_used_mask_ignores_bits_outside_piece_range() {
    let inventory = PieceInventory::from_used_mask(u32::MAX);

    assert_eq!(inventory.used_mask(), ALL_PIECES_MASK);
    assert_eq!(inventory.available_mask(), 0);
    assert_eq!(inventory.used_count(), u32::from(PIECE_COUNT));
    assert_eq!(inventory.available_count(), 0);
    assert!(inventory.is_complete());
}

#[test]
fn mark_used_sets_piece_bit_and_is_idempotent() {
    let mut inventory = PieceInventory::EMPTY;
    let piece = piece_id(7);

    inventory.mark_used(piece);

    assert!(inventory.is_used(piece));
    assert!(!inventory.is_available(piece));
    assert_eq!(inventory.used_mask(), 1u32 << 7);
    assert_eq!(inventory.used_count(), 1);
    assert_eq!(inventory.available_count(), u32::from(PIECE_COUNT) - 1);

    inventory.mark_used(piece);

    assert!(inventory.is_used(piece));
    assert_eq!(inventory.used_mask(), 1u32 << 7);
    assert_eq!(inventory.used_count(), 1);
}

#[test]
fn marked_used_returns_updated_copy_without_mutating_original() {
    let original = PieceInventory::EMPTY;
    let piece = piece_id(3);

    let updated = original.marked_used(piece);

    assert!(!original.is_used(piece));
    assert!(updated.is_used(piece));
}

#[test]
fn all_piece_bits_can_be_marked_used() {
    let mut inventory = PieceInventory::EMPTY;

    for value in 0..PIECE_COUNT {
        let piece = piece_id(value);
        inventory.mark_used(piece);

        assert!(inventory.is_used(piece));
        assert_eq!(inventory.used_count(), u32::from(value) + 1);
    }

    assert_eq!(inventory.used_mask(), ALL_PIECES_MASK);
    assert_eq!(inventory.available_mask(), 0);
    assert!(inventory.is_complete());
}

#[test]
fn inventory_is_copy_eq_hash_and_debug() {
    let first = PieceInventory::EMPTY.marked_used(piece_id(0));
    let duplicate = first;
    let copied = first;
    let second = PieceInventory::EMPTY.marked_used(piece_id(1));

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(format!("{first:?}").contains("PieceInventory"));

    let mut inventories = HashSet::new();
    inventories.insert(first);
    inventories.insert(duplicate);
    inventories.insert(second);

    assert_eq!(inventories.len(), 2);
}
