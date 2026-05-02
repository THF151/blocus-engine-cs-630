use blocus_core::{
    BlocusEngine, BoardIndex, BoardMask, Command, CommandId, DomainError, GameConfig, GameId,
    GameMode, OrientationId, PassCommand, PieceId, PlaceCommand, PlayerColor, PlayerId,
    PlayerSlots, RuleViolation, ScoringMode, SharedColorTurn, TurnOrder, ZobristHash,
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

fn command_id(value: u128) -> CommandId {
    CommandId::from_uuid(uuid(value))
}

fn piece_id(value: u8) -> PieceId {
    PieceId::try_new(value).unwrap_or_else(|_| panic!("piece id {value} should be valid"))
}

fn orientation_id(value: u8) -> OrientationId {
    OrientationId::try_new(value)
        .unwrap_or_else(|_| panic!("orientation id {value} should be valid"))
}

fn index(row: u8, col: u8) -> BoardIndex {
    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|_| panic!("row {row}, col {col} should be valid"))
}

fn three_player_config_shared_green() -> GameConfig {
    let shared = SharedColorTurn::try_new(
        PlayerColor::Green,
        [player_id(1), player_id(2), player_id(3)],
    )
    .unwrap_or_else(|_| panic!("shared turn should be valid"));

    let slots = PlayerSlots::three_player(
        [
            (PlayerColor::Blue, player_id(1)),
            (PlayerColor::Yellow, player_id(2)),
            (PlayerColor::Red, player_id(3)),
        ],
        shared,
    )
    .unwrap_or_else(|_| panic!("three-player slots should be valid"));

    GameConfig::try_new(
        game_id(100),
        GameMode::ThreePlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    )
    .unwrap_or_else(|_| panic!("three-player config should be valid"))
}

fn two_player_config() -> GameConfig {
    let slots = PlayerSlots::two_player(player_id(1), player_id(2))
        .unwrap_or_else(|_| panic!("two-player slots should be valid"));

    GameConfig::try_new(
        game_id(100),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    )
    .unwrap_or_else(|_| panic!("two-player config should be valid"))
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
fn edge_case_three_player_shared_color_accepts_wrong_scheduled_player() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(three_player_config_shared_green());

    state.turn = blocus_core::TurnState::from_parts(PlayerColor::Green, 0, 0);

    let command = Command::Place(PlaceCommand {
        command_id: command_id(1),
        game_id: state.game_id,
        player_id: player_id(2),
        color: PlayerColor::Green,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        anchor: index(19, 0),
    });

    assert_eq!(
        engine.apply(&state, command),
        Err(DomainError::from(RuleViolation::PlayerDoesNotControlColor))
    );
}

#[test]
fn edge_case_four_player_accepts_non_clockwise_turn_order() {
    let non_clockwise_order = TurnOrder::try_new([
        PlayerColor::Blue,
        PlayerColor::Red,
        PlayerColor::Yellow,
        PlayerColor::Green,
    ])
    .unwrap_or_else(|_| panic!("permutation is structurally valid"));

    assert_eq!(
        GameConfig::try_new(
            game_id(101),
            GameMode::FourPlayer,
            ScoringMode::Basic,
            non_clockwise_order,
            four_player_slots(),
        ),
        Err(blocus_core::InputError::InvalidGameConfig)
    );
}

#[test]
fn edge_case_successful_pass_hash_is_not_recomputed() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(two_player_config());

    state.board.place_mask(
        PlayerColor::Yellow,
        BoardMask::from_index(index(0, 0))
            .union(BoardMask::from_index(index(0, 1)))
            .union(BoardMask::from_index(index(1, 0)))
            .union(BoardMask::from_index(index(1, 1))),
    );

    assert_eq!(
        engine.has_any_valid_move(&state, player_id(1), PlayerColor::Blue),
        Ok(false)
    );

    let result = engine
        .apply(
            &state,
            Command::Pass(PassCommand {
                command_id: command_id(1),
                game_id: state.game_id,
                player_id: player_id(1),
                color: PlayerColor::Blue,
            }),
        )
        .unwrap_or_else(|error| panic!("pass should be legal: {error}"));

    assert_eq!(
        result.next_state.hash,
        blocus_core::compute_hash_full(&result.next_state)
    );
    assert_ne!(result.next_state.hash, ZobristHash::ZERO);
}

#[test]
fn edge_case_corrupted_padding_bit_board_state_is_accepted() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(two_player_config());

    let padding_bit_mask = BoardMask::from_lanes([1u128 << 20, 0, 0, 0, 0]);
    state.board.place_mask(PlayerColor::Blue, padding_bit_mask);
    state.hash = blocus_core::compute_hash_full(&state);

    assert!(!state.board.occupied(PlayerColor::Blue).is_playable_subset());

    assert_eq!(
        engine.has_any_valid_move(&state, player_id(1), PlayerColor::Blue),
        Err(DomainError::from(blocus_core::EngineError::CorruptedState))
    );
}
