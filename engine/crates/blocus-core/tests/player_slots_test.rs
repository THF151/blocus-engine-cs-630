use blocus_core::{InputError, PlayerColor, PlayerId, PlayerSlots, SharedColorTurn};
use std::collections::HashSet;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn player_id(value: u128) -> PlayerId {
    PlayerId::from_uuid(uuid(value))
}

fn shared_green() -> SharedColorTurn {
    let Ok(shared) = SharedColorTurn::try_new(
        PlayerColor::Green,
        [player_id(1), player_id(2), player_id(3)],
    ) else {
        panic!("shared color turn should be valid");
    };

    shared
}

#[test]
fn shared_color_turn_rejects_duplicate_players() {
    assert_eq!(
        SharedColorTurn::try_new(
            PlayerColor::Green,
            [player_id(1), player_id(1), player_id(2)]
        ),
        Err(InputError::InvalidGameConfig)
    );

    assert_eq!(
        SharedColorTurn::try_new(
            PlayerColor::Green,
            [player_id(1), player_id(2), player_id(1)]
        ),
        Err(InputError::InvalidGameConfig)
    );

    assert_eq!(
        SharedColorTurn::try_new(
            PlayerColor::Green,
            [player_id(1), player_id(2), player_id(2)]
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn shared_color_turn_exposes_color_players_and_alternating_owner() {
    let shared = shared_green();

    assert_eq!(shared.color(), PlayerColor::Green);
    assert_eq!(shared.players(), [player_id(1), player_id(2), player_id(3)]);

    assert_eq!(shared.owner_at(0), player_id(1));
    assert_eq!(shared.owner_at(1), player_id(2));
    assert_eq!(shared.owner_at(2), player_id(3));
    assert_eq!(shared.owner_at(3), player_id(1));
    assert_eq!(shared.owner_at(4), player_id(2));
    assert_eq!(shared.owner_at(5), player_id(3));

    assert!(shared.contains_player(player_id(1)));
    assert!(shared.contains_player(player_id(2)));
    assert!(shared.contains_player(player_id(3)));
    assert!(!shared.contains_player(player_id(4)));
}

#[test]
fn two_player_slots_assign_blue_red_and_yellow_green() {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    assert_eq!(slots.controller_of(PlayerColor::Blue), Some(player_id(1)));
    assert_eq!(slots.controller_of(PlayerColor::Red), Some(player_id(1)));
    assert_eq!(slots.controller_of(PlayerColor::Yellow), Some(player_id(2)));
    assert_eq!(slots.controller_of(PlayerColor::Green), Some(player_id(2)));

    assert_eq!(
        slots.turn_controller_of(PlayerColor::Blue, 0),
        Some(player_id(1))
    );
    assert_eq!(
        slots.turn_controller_of(PlayerColor::Yellow, 0),
        Some(player_id(2))
    );
    assert_eq!(slots.shared_color(), None);
    assert_eq!(slots.shared_color_turn(), None);

    assert!(slots.can_control_color(player_id(1), PlayerColor::Blue));
    assert!(slots.can_control_color(player_id(1), PlayerColor::Red));
    assert!(!slots.can_control_color(player_id(1), PlayerColor::Yellow));
    assert!(!slots.can_control_color(player_id(3), PlayerColor::Blue));
}

#[test]
fn two_player_slots_reject_same_player_twice() {
    assert_eq!(
        PlayerSlots::two_player(player_id(1), player_id(1)),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn three_player_slots_support_shared_color() {
    let shared = shared_green();

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

    assert_eq!(slots.controller_of(PlayerColor::Blue), Some(player_id(1)));
    assert_eq!(slots.controller_of(PlayerColor::Yellow), Some(player_id(2)));
    assert_eq!(slots.controller_of(PlayerColor::Red), Some(player_id(3)));
    assert_eq!(slots.controller_of(PlayerColor::Green), None);

    assert_eq!(slots.shared_color(), Some(PlayerColor::Green));
    assert_eq!(slots.shared_color_turn(), Some(shared));

    assert_eq!(
        slots.turn_controller_of(PlayerColor::Blue, 0),
        Some(player_id(1))
    );
    assert_eq!(
        slots.turn_controller_of(PlayerColor::Green, 0),
        Some(player_id(1))
    );
    assert_eq!(
        slots.turn_controller_of(PlayerColor::Green, 1),
        Some(player_id(2))
    );
    assert_eq!(
        slots.turn_controller_of(PlayerColor::Green, 2),
        Some(player_id(3))
    );
    assert_eq!(
        slots.turn_controller_of(PlayerColor::Green, 3),
        Some(player_id(1))
    );

    assert!(slots.can_control_color(player_id(1), PlayerColor::Green));
    assert!(slots.can_control_color(player_id(2), PlayerColor::Green));
    assert!(slots.can_control_color(player_id(3), PlayerColor::Green));
    assert!(!slots.can_control_color(player_id(4), PlayerColor::Green));
}

#[test]
fn three_player_slots_reject_duplicate_owned_color() {
    let shared = shared_green();

    assert_eq!(
        PlayerSlots::three_player(
            [
                (PlayerColor::Blue, player_id(1)),
                (PlayerColor::Blue, player_id(2)),
                (PlayerColor::Red, player_id(3)),
            ],
            shared,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn three_player_slots_reject_shared_color_as_owned_color() {
    let shared = shared_green();

    assert_eq!(
        PlayerSlots::three_player(
            [
                (PlayerColor::Blue, player_id(1)),
                (PlayerColor::Yellow, player_id(2)),
                (PlayerColor::Green, player_id(3)),
            ],
            shared,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn three_player_slots_reject_duplicate_owned_player() {
    let shared = shared_green();

    assert_eq!(
        PlayerSlots::three_player(
            [
                (PlayerColor::Blue, player_id(1)),
                (PlayerColor::Yellow, player_id(1)),
                (PlayerColor::Red, player_id(3)),
            ],
            shared,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn three_player_slots_reject_shared_cycle_player_not_in_owned_players() {
    let Ok(shared) = SharedColorTurn::try_new(
        PlayerColor::Green,
        [player_id(1), player_id(2), player_id(4)],
    ) else {
        panic!("shared color turn should be structurally valid");
    };

    assert_eq!(
        PlayerSlots::three_player(
            [
                (PlayerColor::Blue, player_id(1)),
                (PlayerColor::Yellow, player_id(2)),
                (PlayerColor::Red, player_id(3)),
            ],
            shared,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn three_player_slots_allow_any_color_to_be_shared() {
    let Ok(shared) = SharedColorTurn::try_new(
        PlayerColor::Blue,
        [player_id(1), player_id(2), player_id(3)],
    ) else {
        panic!("shared color turn should be valid");
    };

    let Ok(slots) = PlayerSlots::three_player(
        [
            (PlayerColor::Yellow, player_id(1)),
            (PlayerColor::Red, player_id(2)),
            (PlayerColor::Green, player_id(3)),
        ],
        shared,
    ) else {
        panic!("three-player slots should be valid with blue shared");
    };

    assert_eq!(slots.shared_color(), Some(PlayerColor::Blue));
    assert_eq!(slots.controller_of(PlayerColor::Blue), None);
    assert_eq!(
        slots.turn_controller_of(PlayerColor::Blue, 2),
        Some(player_id(3))
    );
}

#[test]
fn four_player_slots_assign_one_player_per_color() {
    let Ok(slots) = PlayerSlots::four_player([
        (PlayerColor::Blue, player_id(1)),
        (PlayerColor::Yellow, player_id(2)),
        (PlayerColor::Red, player_id(3)),
        (PlayerColor::Green, player_id(4)),
    ]) else {
        panic!("four-player slots should be valid");
    };

    assert_eq!(slots.controller_of(PlayerColor::Blue), Some(player_id(1)));
    assert_eq!(slots.controller_of(PlayerColor::Yellow), Some(player_id(2)));
    assert_eq!(slots.controller_of(PlayerColor::Red), Some(player_id(3)));
    assert_eq!(slots.controller_of(PlayerColor::Green), Some(player_id(4)));
    assert_eq!(slots.shared_color(), None);
    assert_eq!(slots.shared_color_turn(), None);

    assert!(slots.can_control_color(player_id(1), PlayerColor::Blue));
    assert!(!slots.can_control_color(player_id(1), PlayerColor::Yellow));
}

#[test]
fn four_player_slots_reject_duplicate_color() {
    assert_eq!(
        PlayerSlots::four_player([
            (PlayerColor::Blue, player_id(1)),
            (PlayerColor::Blue, player_id(2)),
            (PlayerColor::Red, player_id(3)),
            (PlayerColor::Green, player_id(4)),
        ]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn four_player_slots_reject_duplicate_player() {
    assert_eq!(
        PlayerSlots::four_player([
            (PlayerColor::Blue, player_id(1)),
            (PlayerColor::Yellow, player_id(1)),
            (PlayerColor::Red, player_id(3)),
            (PlayerColor::Green, player_id(4)),
        ]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn four_player_slots_reject_missing_color() {
    assert_eq!(
        PlayerSlots::four_player([
            (PlayerColor::Blue, player_id(1)),
            (PlayerColor::Yellow, player_id(2)),
            (PlayerColor::Red, player_id(3)),
            (PlayerColor::Red, player_id(4)),
        ]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn player_slots_expose_controllers_array() {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    assert_eq!(
        slots.controllers(),
        [
            Some(player_id(1)),
            Some(player_id(2)),
            Some(player_id(1)),
            Some(player_id(2)),
        ]
    );
}

#[test]
fn player_slots_are_copy_eq_hash_and_debug() {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    let copied = slots;
    let duplicate = slots;

    let Ok(other) = PlayerSlots::two_player(player_id(3), player_id(4)) else {
        panic!("two-player slots should be valid");
    };

    assert_eq!(slots, copied);
    assert_eq!(slots, duplicate);
    assert_ne!(slots, other);
    assert!(format!("{slots:?}").contains("PlayerSlots"));

    let mut slot_set = HashSet::new();
    slot_set.insert(slots);
    slot_set.insert(duplicate);
    slot_set.insert(other);

    assert_eq!(slot_set.len(), 2);
}

#[test]
fn shared_color_turn_is_copy_eq_hash_and_debug() {
    let shared = shared_green();
    let copied = shared;
    let duplicate = shared;

    let Ok(other) = SharedColorTurn::try_new(
        PlayerColor::Blue,
        [player_id(1), player_id(2), player_id(3)],
    ) else {
        panic!("shared color turn should be valid");
    };

    assert_eq!(shared, copied);
    assert_eq!(shared, duplicate);
    assert_ne!(shared, other);
    assert!(format!("{shared:?}").contains("SharedColorTurn"));

    let mut shared_set = HashSet::new();
    shared_set.insert(shared);
    shared_set.insert(duplicate);
    shared_set.insert(other);

    assert_eq!(shared_set.len(), 2);
}
