use blocus_core::{InputError, PLAYER_COLOR_COUNT, PlayerColor, TurnOrder, TurnOrderPolicy};
use std::collections::HashSet;

const CONST_ALL: [PlayerColor; PLAYER_COLOR_COUNT] = PlayerColor::ALL;
const CONST_OFFICIAL_FIXED_ARRAY: [PlayerColor; PLAYER_COLOR_COUNT] =
    PlayerColor::OFFICIAL_FIXED_TURN_ORDER;

const CONST_BLUE_INDEX: usize = PlayerColor::Blue.index();
const CONST_YELLOW_INDEX: usize = PlayerColor::Yellow.index();
const CONST_RED_INDEX: usize = PlayerColor::Red.index();
const CONST_GREEN_INDEX: usize = PlayerColor::Green.index();

const CONST_FROM_INDEX_ZERO: Option<PlayerColor> = PlayerColor::from_index(0);
const CONST_FROM_INDEX_THREE: Option<PlayerColor> = PlayerColor::from_index(3);
const CONST_FROM_INDEX_FOUR: Option<PlayerColor> = PlayerColor::from_index(4);

const CONST_BLUE_NEXT_FIXED: PlayerColor = PlayerColor::Blue.next_in_official_fixed_order();
const CONST_YELLOW_NEXT_FIXED: PlayerColor = PlayerColor::Yellow.next_in_official_fixed_order();
const CONST_RED_NEXT_FIXED: PlayerColor = PlayerColor::Red.next_in_official_fixed_order();
const CONST_GREEN_NEXT_FIXED: PlayerColor = PlayerColor::Green.next_in_official_fixed_order();

const CONST_BLUE_NAME: &str = PlayerColor::Blue.as_str();
const CONST_YELLOW_NAME: &str = PlayerColor::Yellow.as_str();
const CONST_RED_NAME: &str = PlayerColor::Red.as_str();
const CONST_GREEN_NAME: &str = PlayerColor::Green.as_str();

const CONST_OFFICIAL_TURN_ORDER: TurnOrder = TurnOrder::OFFICIAL_FIXED;
const CONST_OFFICIAL_TURN_ORDER_FIRST: PlayerColor = TurnOrder::OFFICIAL_FIXED.first();
const CONST_OFFICIAL_TURN_ORDER_COLORS: [PlayerColor; PLAYER_COLOR_COUNT] =
    TurnOrder::OFFICIAL_FIXED.colors();
const CONST_OFFICIAL_TURN_ORDER_IS_FIXED: bool = TurnOrder::OFFICIAL_FIXED.is_official_fixed();

const CONST_FIXED_POLICY_OK: Result<(), InputError> =
    TurnOrder::OFFICIAL_FIXED.validate_for_policy(TurnOrderPolicy::OfficialFixed);
const CONST_FLEXIBLE_POLICY_OK: Result<(), InputError> =
    TurnOrder::OFFICIAL_FIXED.validate_for_policy(TurnOrderPolicy::Flexible);

fn custom_turn_order() -> TurnOrder {
    let Ok(order) = TurnOrder::try_new([
        PlayerColor::Red,
        PlayerColor::Green,
        PlayerColor::Blue,
        PlayerColor::Yellow,
    ]) else {
        panic!("custom turn order should be structurally valid");
    };

    order
}

fn assert_valid_turn_order(colors: [PlayerColor; PLAYER_COLOR_COUNT]) {
    let Ok(order) = TurnOrder::try_new(colors) else {
        panic!("turn order {colors:?} should be valid");
    };

    assert_eq!(order.colors(), colors);
    assert_eq!(order.first(), colors[0]);
    assert_eq!(
        order.is_official_fixed(),
        colors == PlayerColor::OFFICIAL_FIXED_TURN_ORDER
    );

    for index in 0..PLAYER_COLOR_COUNT {
        let color = colors[index];

        assert_eq!(order.position_of(color), index);
        assert_eq!(
            order.next_after(color),
            colors[(index + 1) % PLAYER_COLOR_COUNT]
        );
    }
}

#[test]
fn player_color_count_matches_classic_blokus_colors() {
    assert_eq!(PLAYER_COLOR_COUNT, 4);
}

#[test]
fn all_colors_are_in_stable_storage_order() {
    assert_eq!(
        PlayerColor::ALL,
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ]
    );

    assert_eq!(CONST_ALL, PlayerColor::ALL);
}

#[test]
fn official_fixed_turn_order_is_blue_yellow_red_green() {
    assert_eq!(
        PlayerColor::OFFICIAL_FIXED_TURN_ORDER,
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ]
    );

    assert_eq!(
        CONST_OFFICIAL_FIXED_ARRAY,
        PlayerColor::OFFICIAL_FIXED_TURN_ORDER
    );
}

#[test]
fn stable_storage_order_contains_each_color_once() {
    let colors = PlayerColor::ALL.into_iter().collect::<HashSet<_>>();

    assert_eq!(colors.len(), PLAYER_COLOR_COUNT);
    assert!(colors.contains(&PlayerColor::Blue));
    assert!(colors.contains(&PlayerColor::Yellow));
    assert!(colors.contains(&PlayerColor::Red));
    assert!(colors.contains(&PlayerColor::Green));
}

#[test]
fn player_color_indices_are_stable_storage_indices() {
    assert_eq!(PlayerColor::Blue.index(), 0);
    assert_eq!(PlayerColor::Yellow.index(), 1);
    assert_eq!(PlayerColor::Red.index(), 2);
    assert_eq!(PlayerColor::Green.index(), 3);

    assert_eq!(CONST_BLUE_INDEX, 0);
    assert_eq!(CONST_YELLOW_INDEX, 1);
    assert_eq!(CONST_RED_INDEX, 2);
    assert_eq!(CONST_GREEN_INDEX, 3);
}

#[test]
fn player_color_from_index_accepts_only_valid_storage_indices() {
    assert_eq!(PlayerColor::from_index(0), Some(PlayerColor::Blue));
    assert_eq!(PlayerColor::from_index(1), Some(PlayerColor::Yellow));
    assert_eq!(PlayerColor::from_index(2), Some(PlayerColor::Red));
    assert_eq!(PlayerColor::from_index(3), Some(PlayerColor::Green));
    assert_eq!(PlayerColor::from_index(4), None);
    assert_eq!(PlayerColor::from_index(usize::MAX), None);

    assert_eq!(CONST_FROM_INDEX_ZERO, Some(PlayerColor::Blue));
    assert_eq!(CONST_FROM_INDEX_THREE, Some(PlayerColor::Green));
    assert_eq!(CONST_FROM_INDEX_FOUR, None);
}

#[test]
fn every_color_round_trips_through_storage_index() {
    for color in PlayerColor::ALL {
        assert_eq!(PlayerColor::from_index(color.index()), Some(color));
    }
}

#[test]
fn every_valid_storage_index_round_trips_through_color() {
    for index in 0..PLAYER_COLOR_COUNT {
        let Some(color) = PlayerColor::from_index(index) else {
            panic!("storage index {index} should map to a color");
        };

        assert_eq!(color.index(), index);
    }
}

#[test]
fn official_fixed_order_advances_blue_yellow_red_green() {
    assert_eq!(
        PlayerColor::Blue.next_in_official_fixed_order(),
        PlayerColor::Yellow
    );
    assert_eq!(
        PlayerColor::Yellow.next_in_official_fixed_order(),
        PlayerColor::Red
    );
    assert_eq!(
        PlayerColor::Red.next_in_official_fixed_order(),
        PlayerColor::Green
    );
    assert_eq!(
        PlayerColor::Green.next_in_official_fixed_order(),
        PlayerColor::Blue
    );

    assert_eq!(CONST_BLUE_NEXT_FIXED, PlayerColor::Yellow);
    assert_eq!(CONST_YELLOW_NEXT_FIXED, PlayerColor::Red);
    assert_eq!(CONST_RED_NEXT_FIXED, PlayerColor::Green);
    assert_eq!(CONST_GREEN_NEXT_FIXED, PlayerColor::Blue);
}

#[test]
fn player_color_api_names_are_stable() {
    assert_eq!(PlayerColor::Blue.as_str(), "blue");
    assert_eq!(PlayerColor::Yellow.as_str(), "yellow");
    assert_eq!(PlayerColor::Red.as_str(), "red");
    assert_eq!(PlayerColor::Green.as_str(), "green");

    assert_eq!(PlayerColor::Blue.to_string(), "blue");
    assert_eq!(PlayerColor::Yellow.to_string(), "yellow");
    assert_eq!(PlayerColor::Red.to_string(), "red");
    assert_eq!(PlayerColor::Green.to_string(), "green");

    assert_eq!(CONST_BLUE_NAME, "blue");
    assert_eq!(CONST_YELLOW_NAME, "yellow");
    assert_eq!(CONST_RED_NAME, "red");
    assert_eq!(CONST_GREEN_NAME, "green");
}

#[test]
fn player_color_is_copy_comparable_ordered_and_hashable() {
    let blue = PlayerColor::Blue;
    let duplicate_blue = PlayerColor::Blue;
    let yellow = PlayerColor::Yellow;
    let copied = blue;

    assert_eq!(blue, copied);
    assert_eq!(blue, duplicate_blue);
    assert_ne!(blue, yellow);
    assert!(blue < yellow);

    let mut colors = HashSet::new();
    colors.insert(blue);
    colors.insert(duplicate_blue);
    colors.insert(yellow);

    assert_eq!(colors.len(), 2);
    assert!(colors.contains(&PlayerColor::Blue));
    assert!(colors.contains(&PlayerColor::Yellow));
}

#[test]
fn turn_order_official_fixed_matches_rulebook_fixed_order() {
    let order = TurnOrder::OFFICIAL_FIXED;

    assert_eq!(
        order.colors(),
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ]
    );
    assert_eq!(order.first(), PlayerColor::Blue);
    assert!(order.is_official_fixed());

    assert_eq!(CONST_OFFICIAL_TURN_ORDER, TurnOrder::OFFICIAL_FIXED);
    assert_eq!(CONST_OFFICIAL_TURN_ORDER_FIRST, PlayerColor::Blue);
    assert_eq!(
        CONST_OFFICIAL_TURN_ORDER_COLORS,
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ]
    );
    assert_eq!(
        CONST_OFFICIAL_TURN_ORDER_IS_FIXED,
        TurnOrder::OFFICIAL_FIXED.is_official_fixed()
    );
}

#[test]
fn turn_order_default_is_official_fixed() {
    assert_eq!(TurnOrder::default(), TurnOrder::OFFICIAL_FIXED);
}

#[test]
fn turn_order_try_new_accepts_representative_custom_permutation() {
    let order = custom_turn_order();

    assert_eq!(
        order.colors(),
        [
            PlayerColor::Red,
            PlayerColor::Green,
            PlayerColor::Blue,
            PlayerColor::Yellow,
        ]
    );
    assert_eq!(order.first(), PlayerColor::Red);
    assert!(!order.is_official_fixed());
}

#[test]
fn turn_order_try_new_accepts_exactly_permutations_of_all_colors() {
    for first in PlayerColor::ALL {
        for second in PlayerColor::ALL {
            for third in PlayerColor::ALL {
                for fourth in PlayerColor::ALL {
                    let colors = [first, second, third, fourth];
                    let unique_count = colors.into_iter().collect::<HashSet<_>>().len();
                    let result = TurnOrder::try_new(colors);

                    if unique_count == PLAYER_COLOR_COUNT {
                        assert_valid_turn_order(colors);
                    } else {
                        assert_eq!(result, Err(InputError::InvalidGameConfig));
                    }
                }
            }
        }
    }
}

#[test]
fn turn_order_try_new_rejects_duplicate_color() {
    assert_eq!(
        TurnOrder::try_new([
            PlayerColor::Blue,
            PlayerColor::Blue,
            PlayerColor::Red,
            PlayerColor::Green,
        ]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn turn_order_try_new_rejects_missing_color() {
    assert_eq!(
        TurnOrder::try_new([
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Red,
        ]),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn turn_order_next_after_uses_game_specific_cycle() {
    let order = custom_turn_order();

    assert_eq!(order.next_after(PlayerColor::Red), PlayerColor::Green);
    assert_eq!(order.next_after(PlayerColor::Green), PlayerColor::Blue);
    assert_eq!(order.next_after(PlayerColor::Blue), PlayerColor::Yellow);
    assert_eq!(order.next_after(PlayerColor::Yellow), PlayerColor::Red);
}

#[test]
fn turn_order_position_of_uses_game_specific_order_not_storage_index() {
    let order = custom_turn_order();

    assert_eq!(PlayerColor::Red.index(), 2);
    assert_eq!(order.position_of(PlayerColor::Red), 0);

    assert_eq!(PlayerColor::Blue.index(), 0);
    assert_eq!(order.position_of(PlayerColor::Blue), 2);
}

#[test]
fn flexible_policy_allows_official_fixed_order() {
    assert_eq!(
        TurnOrder::OFFICIAL_FIXED.validate_for_policy(TurnOrderPolicy::Flexible),
        Ok(())
    );
}

#[test]
fn flexible_policy_allows_custom_order() {
    let custom_order = custom_turn_order();

    assert_eq!(
        custom_order.validate_for_policy(TurnOrderPolicy::Flexible),
        Ok(())
    );
}

#[test]
fn official_fixed_policy_requires_official_fixed_order() {
    assert_eq!(
        TurnOrder::OFFICIAL_FIXED.validate_for_policy(TurnOrderPolicy::OfficialFixed),
        Ok(())
    );

    let custom_order = custom_turn_order();

    assert_eq!(
        custom_order.validate_for_policy(TurnOrderPolicy::OfficialFixed),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn turn_order_policy_is_copy_comparable_ordered_and_hashable() {
    let flexible = TurnOrderPolicy::Flexible;
    let duplicate_flexible = TurnOrderPolicy::Flexible;
    let fixed = TurnOrderPolicy::OfficialFixed;
    let copied = flexible;

    assert_eq!(flexible, copied);
    assert_eq!(flexible, duplicate_flexible);
    assert_ne!(flexible, fixed);
    assert!(flexible < fixed);

    let mut policies = HashSet::new();
    policies.insert(flexible);
    policies.insert(duplicate_flexible);
    policies.insert(fixed);

    assert_eq!(policies.len(), 2);
    assert!(policies.contains(&TurnOrderPolicy::Flexible));
    assert!(policies.contains(&TurnOrderPolicy::OfficialFixed));
}

#[test]
fn turn_order_is_copy_comparable_ordered_and_hashable() {
    let official = TurnOrder::OFFICIAL_FIXED;
    let duplicate_official = TurnOrder::OFFICIAL_FIXED;
    let custom = custom_turn_order();
    let copied = official;

    assert_eq!(official, copied);
    assert_eq!(official, duplicate_official);
    assert_ne!(official, custom);

    let mut orders = HashSet::new();
    orders.insert(official);
    orders.insert(duplicate_official);
    orders.insert(custom);

    assert_eq!(orders.len(), 2);
    assert!(orders.contains(&TurnOrder::OFFICIAL_FIXED));
    assert!(orders.contains(&custom));
}

#[test]
fn const_context_apis_work_for_player_color() {
    assert_eq!(
        CONST_ALL,
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ]
    );
    assert_eq!(
        CONST_OFFICIAL_FIXED_ARRAY,
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ]
    );

    assert_eq!(CONST_BLUE_INDEX, 0);
    assert_eq!(CONST_YELLOW_INDEX, 1);
    assert_eq!(CONST_RED_INDEX, 2);
    assert_eq!(CONST_GREEN_INDEX, 3);

    assert_eq!(CONST_FROM_INDEX_ZERO, Some(PlayerColor::Blue));
    assert_eq!(CONST_FROM_INDEX_THREE, Some(PlayerColor::Green));
    assert_eq!(CONST_FROM_INDEX_FOUR, None);

    assert_eq!(CONST_BLUE_NEXT_FIXED, PlayerColor::Yellow);
    assert_eq!(CONST_YELLOW_NEXT_FIXED, PlayerColor::Red);
    assert_eq!(CONST_RED_NEXT_FIXED, PlayerColor::Green);
    assert_eq!(CONST_GREEN_NEXT_FIXED, PlayerColor::Blue);

    assert_eq!(CONST_BLUE_NAME, "blue");
    assert_eq!(CONST_YELLOW_NAME, "yellow");
    assert_eq!(CONST_RED_NAME, "red");
    assert_eq!(CONST_GREEN_NAME, "green");
}

#[test]
fn const_context_apis_work_for_turn_order() {
    assert_eq!(CONST_OFFICIAL_TURN_ORDER, TurnOrder::OFFICIAL_FIXED);
    assert_eq!(CONST_OFFICIAL_TURN_ORDER_FIRST, PlayerColor::Blue);
    assert_eq!(
        CONST_OFFICIAL_TURN_ORDER_COLORS,
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ]
    );
    assert_eq!(
        CONST_OFFICIAL_TURN_ORDER_IS_FIXED,
        TurnOrder::OFFICIAL_FIXED.is_official_fixed()
    );
}

#[test]
fn const_context_apis_work_for_turn_order_policy() {
    assert_eq!(CONST_FIXED_POLICY_OK, Ok(()));
    assert_eq!(CONST_FLEXIBLE_POLICY_OK, Ok(()));
}
