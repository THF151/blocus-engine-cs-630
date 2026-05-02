use blocus_core::{
    BOARD_LANES, BOARD_SIZE, BoardIndex, BoardMask, GameConfig, GameId, GameMode, InputError,
    PlayerColor, PlayerId, PlayerSlots, ScoringMode, TurnOrder,
};
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

fn four_player_slots() -> PlayerSlots {
    PlayerSlots::four_player([
        (PlayerColor::Blue, player_id(1)),
        (PlayerColor::Yellow, player_id(2)),
        (PlayerColor::Red, player_id(3)),
        (PlayerColor::Green, player_id(4)),
    ])
    .unwrap_or_else(|_| panic!("four-player slots should be valid"))
}

#[test]
fn board_mask_try_from_lanes_rejects_padding_bits() {
    let lanes = [1u128 << BOARD_SIZE, 0, 0, 0, 0];

    assert_eq!(
        BoardMask::try_from_lanes(lanes),
        Err(InputError::InvalidBoardIndex)
    );
}

#[test]
fn board_mask_try_from_lanes_accepts_only_playable_bits() {
    let index =
        BoardIndex::from_row_col(0, 0).unwrap_or_else(|_| panic!("top-left should be valid"));

    let mut lanes = [0u128; BOARD_LANES];
    lanes[index.lane()] = index.lane_bit();

    let mask = BoardMask::try_from_lanes(lanes)
        .unwrap_or_else(|error| panic!("playable lanes should be accepted: {error}"));

    assert!(mask.contains(index));
    assert_eq!(mask.count(), 1);
    assert!(mask.is_playable_subset());
}

#[test]
fn four_player_config_accepts_any_clockwise_rotation_as_first_player_choice() {
    let slots = four_player_slots();

    let rotations = [
        [
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
        ],
        [
            PlayerColor::Yellow,
            PlayerColor::Red,
            PlayerColor::Green,
            PlayerColor::Blue,
        ],
        [
            PlayerColor::Red,
            PlayerColor::Green,
            PlayerColor::Blue,
            PlayerColor::Yellow,
        ],
        [
            PlayerColor::Green,
            PlayerColor::Blue,
            PlayerColor::Yellow,
            PlayerColor::Red,
        ],
    ];

    for colors in rotations {
        let order = TurnOrder::try_new(colors)
            .unwrap_or_else(|_| panic!("rotation should be structurally valid"));

        assert!(
            GameConfig::try_new(
                game_id(99),
                GameMode::FourPlayer,
                ScoringMode::Basic,
                order,
                slots,
            )
            .is_ok(),
            "clockwise rotation {colors:?} should be accepted"
        );
    }
}

#[test]
fn four_player_config_rejects_non_clockwise_turn_order() {
    let slots = four_player_slots();

    let non_clockwise = TurnOrder::try_new([
        PlayerColor::Blue,
        PlayerColor::Red,
        PlayerColor::Yellow,
        PlayerColor::Green,
    ])
    .unwrap_or_else(|_| panic!("permutation is structurally valid"));

    assert_eq!(
        GameConfig::try_new(
            game_id(100),
            GameMode::FourPlayer,
            ScoringMode::Basic,
            non_clockwise,
            slots,
        ),
        Err(InputError::InvalidGameConfig)
    );
}
