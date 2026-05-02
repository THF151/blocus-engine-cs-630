use blocus_core::{
    GameConfig, GameId, GameMode, InputError, PlayerColor, PlayerId, PlayerSlots, ScoringMode,
    SharedColorTurn, TurnOrder, TurnOrderPolicy,
};
use std::collections::HashSet;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn game_id(value: u128) -> GameId {
    GameId::from_uuid(uuid(value))
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

#[test]
fn game_mode_reports_turn_order_policy_and_player_count() {
    assert_eq!(
        GameMode::TwoPlayer.turn_order_policy(),
        TurnOrderPolicy::OfficialFixed
    );
    assert_eq!(
        GameMode::ThreePlayer.turn_order_policy(),
        TurnOrderPolicy::OfficialFixed
    );
    assert_eq!(
        GameMode::FourPlayer.turn_order_policy(),
        TurnOrderPolicy::ClockwiseRotation
    );

    assert_eq!(GameMode::TwoPlayer.player_count(), 2);
    assert_eq!(GameMode::ThreePlayer.player_count(), 3);
    assert_eq!(GameMode::FourPlayer.player_count(), 4);
}

#[test]
fn game_mode_is_copy_ordered_and_hashable() {
    let two = GameMode::TwoPlayer;
    let duplicate_two = GameMode::TwoPlayer;
    let three = GameMode::ThreePlayer;
    let copied = two;

    assert_eq!(two, copied);
    assert_eq!(two, duplicate_two);
    assert_ne!(two, three);
    assert!(two < three);

    let mut modes = HashSet::new();
    modes.insert(two);
    modes.insert(duplicate_two);
    modes.insert(three);

    assert_eq!(modes.len(), 2);
    assert!(modes.contains(&GameMode::TwoPlayer));
    assert!(modes.contains(&GameMode::ThreePlayer));
}

#[test]
fn game_config_accepts_two_player_fixed_order() {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    let Ok(config) = GameConfig::try_new(
        game_id(10),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    ) else {
        panic!("two-player config should be valid");
    };

    assert_eq!(config.game_id(), game_id(10));
    assert_eq!(config.mode(), GameMode::TwoPlayer);
    assert_eq!(config.scoring(), ScoringMode::Basic);
    assert_eq!(config.turn_order(), TurnOrder::OFFICIAL_FIXED);
    assert_eq!(config.player_slots(), slots);
}

#[test]
fn game_config_accepts_three_player_fixed_order() {
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

    let Ok(config) = GameConfig::try_new(
        game_id(11),
        GameMode::ThreePlayer,
        ScoringMode::Advanced,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    ) else {
        panic!("three-player config should be valid");
    };

    assert_eq!(config.game_id(), game_id(11));
    assert_eq!(config.mode(), GameMode::ThreePlayer);
    assert_eq!(config.scoring(), ScoringMode::Advanced);
    assert_eq!(config.turn_order(), TurnOrder::OFFICIAL_FIXED);
    assert_eq!(config.player_slots(), slots);
}

#[test]
fn game_config_accepts_four_player_custom_order() {
    let Ok(slots) = PlayerSlots::four_player([
        (PlayerColor::Blue, player_id(1)),
        (PlayerColor::Yellow, player_id(2)),
        (PlayerColor::Red, player_id(3)),
        (PlayerColor::Green, player_id(4)),
    ]) else {
        panic!("four-player slots should be valid");
    };

    let turn_order = custom_turn_order();

    let Ok(config) = GameConfig::try_new(
        game_id(12),
        GameMode::FourPlayer,
        ScoringMode::Basic,
        turn_order,
        slots,
    ) else {
        panic!("four-player custom-order config should be valid");
    };

    assert_eq!(config.mode(), GameMode::FourPlayer);
    assert_eq!(config.turn_order(), turn_order);
}

#[test]
fn game_config_rejects_mode_and_slots_mismatch() {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    assert_eq!(
        GameConfig::try_new(
            game_id(13),
            GameMode::FourPlayer,
            ScoringMode::Basic,
            TurnOrder::OFFICIAL_FIXED,
            slots,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn game_config_rejects_custom_order_for_two_player_mode() {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    assert_eq!(
        GameConfig::try_new(
            game_id(14),
            GameMode::TwoPlayer,
            ScoringMode::Basic,
            custom_turn_order(),
            slots,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn game_config_rejects_custom_order_for_three_player_mode() {
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

    assert_eq!(
        GameConfig::try_new(
            game_id(15),
            GameMode::ThreePlayer,
            ScoringMode::Basic,
            custom_turn_order(),
            slots,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn game_config_is_copy_eq_hash_and_debug() {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    let Ok(config) = GameConfig::try_new(
        game_id(16),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    ) else {
        panic!("two-player config should be valid");
    };

    let copied = config;
    let duplicate = config;

    let Ok(other_slots) = PlayerSlots::two_player(player_id(3), player_id(4)) else {
        panic!("two-player slots should be valid");
    };

    let Ok(other) = GameConfig::try_new(
        game_id(17),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        other_slots,
    ) else {
        panic!("two-player config should be valid");
    };

    assert_eq!(config, copied);
    assert_eq!(config, duplicate);
    assert_ne!(config, other);
    assert!(format!("{config:?}").contains("GameConfig"));

    let mut configs = HashSet::new();
    configs.insert(config);
    configs.insert(duplicate);
    configs.insert(other);

    assert_eq!(configs.len(), 2);
}
