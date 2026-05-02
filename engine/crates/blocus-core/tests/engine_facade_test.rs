use blocus_core::{
    BlocusEngine, BoardMask, BoardState, Command, CommandId, DomainError, GameConfig, GameMode,
    GameStatus, PassCommand, PieceId, PlayerColor, PlayerId, PlayerSlots, RuleViolation,
    ScoringMode, StateSchemaVersion, StateVersion, TurnOrder, TurnState, ZobristHash,
};
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn player(value: u128) -> PlayerId {
    PlayerId::from_uuid(uuid(value))
}

fn command_id(value: u128) -> CommandId {
    CommandId::from_uuid(uuid(value))
}

fn piece_id(value: u8) -> PieceId {
    let Ok(piece_id) = PieceId::try_new(value) else {
        panic!("piece id {value} should be valid");
    };

    piece_id
}

fn game_config(
    game_id_value: u128,
    mode: GameMode,
    scoring: ScoringMode,
    turn_order: TurnOrder,
    player_slots: PlayerSlots,
) -> GameConfig {
    let Ok(config) = GameConfig::try_new(
        uuid(game_id_value).into(),
        mode,
        scoring,
        turn_order,
        player_slots,
    ) else {
        panic!("test game config should be valid");
    };

    config
}

fn two_player_slots() -> PlayerSlots {
    let Ok(slots) = PlayerSlots::two_player(player(1), player(2)) else {
        panic!("two-player slots should be valid");
    };

    slots
}

fn three_player_slots() -> PlayerSlots {
    let Ok(shared_color_turn) = blocus_core::SharedColorTurn::try_new(
        PlayerColor::Green,
        [player(1), player(2), player(3)],
    ) else {
        panic!("shared color turn should be valid");
    };

    let Ok(slots) = PlayerSlots::three_player(
        [
            (PlayerColor::Blue, player(1)),
            (PlayerColor::Yellow, player(2)),
            (PlayerColor::Red, player(3)),
        ],
        shared_color_turn,
    ) else {
        panic!("three-player slots should be valid");
    };

    slots
}

fn four_player_slots() -> PlayerSlots {
    let Ok(slots) = PlayerSlots::four_player([
        (PlayerColor::Blue, player(1)),
        (PlayerColor::Yellow, player(2)),
        (PlayerColor::Red, player(3)),
        (PlayerColor::Green, player(4)),
    ]) else {
        panic!("four-player slots should be valid");
    };

    slots
}

fn two_player_config() -> GameConfig {
    game_config(
        100,
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        two_player_slots(),
    )
}

fn three_player_config() -> GameConfig {
    game_config(
        101,
        GameMode::ThreePlayer,
        ScoringMode::Advanced,
        TurnOrder::OFFICIAL_FIXED,
        three_player_slots(),
    )
}

fn four_player_config() -> GameConfig {
    let Ok(turn_order) = TurnOrder::try_new([
        PlayerColor::Red,
        PlayerColor::Green,
        PlayerColor::Blue,
        PlayerColor::Yellow,
    ]) else {
        panic!("custom turn order should be valid");
    };

    game_config(
        102,
        GameMode::FourPlayer,
        ScoringMode::Basic,
        turn_order,
        four_player_slots(),
    )
}

fn assert_initial_state_matches_config(config: GameConfig) {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(config);

    assert_eq!(state.schema_version, StateSchemaVersion::CURRENT);
    assert_eq!(state.game_id, config.game_id());
    assert_eq!(state.mode, config.mode());
    assert_eq!(state.scoring, config.scoring());
    assert_eq!(state.board, BoardState::EMPTY);
    assert_eq!(state.turn_order, config.turn_order());
    assert_eq!(state.player_slots, config.player_slots());
    assert_eq!(state.turn, TurnState::new(config.turn_order()));
    assert_eq!(state.status, GameStatus::InProgress);
    assert_eq!(state.version, StateVersion::INITIAL);
    assert_eq!(state.hash, ZobristHash::ZERO);

    for inventory in state.inventories {
        assert_eq!(inventory.used_count(), 0);
        assert!(inventory.is_available(piece_id(0)));
        assert!(inventory.is_available(piece_id(20)));
    }
}

#[test]
fn engine_health_still_reports_native_linkage() {
    assert!(blocus_core::engine_health());
}

#[test]
fn engine_facade_is_stateless_copy_debug_default_and_comparable() {
    let engine = BlocusEngine::new();
    let copied = engine;

    assert_eq!(engine, copied);
    assert_eq!(engine, BlocusEngine::new());
    assert!(format!("{engine:?}").contains("BlocusEngine"));
}

#[test]
fn engine_facade_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<BlocusEngine>();
}

#[test]
fn initialize_game_builds_empty_two_player_state() {
    assert_initial_state_matches_config(two_player_config());
}

#[test]
fn initialize_game_builds_empty_three_player_state() {
    assert_initial_state_matches_config(three_player_config());
}

#[test]
fn initialize_game_builds_empty_four_player_state_with_custom_turn_order() {
    assert_initial_state_matches_config(four_player_config());
}

#[test]
fn apply_pass_rejects_pass_while_a_legal_move_exists() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let command = Command::Pass(PassCommand {
        command_id: command_id(1),
        game_id: state.game_id,
        player_id: player(1),
        color: PlayerColor::Blue,
    });

    assert_eq!(
        engine.apply(&state, command),
        Err(DomainError::from(
            blocus_core::RuleViolation::PassNotAllowedBecauseMoveExists
        ))
    );
}

#[test]
fn valid_moves_iter_returns_real_legal_moves() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .valid_moves_iter(&state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
        .take(1)
        .collect::<Vec<_>>();

    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].piece_id, piece_id(0));
    assert_eq!(moves[0].score_delta, 1);
}

#[test]
fn valid_moves_iter_returns_no_moves_for_wrong_turn() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .valid_moves_iter(&state, player(2), PlayerColor::Yellow)
        .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
        .collect::<Vec<_>>();

    assert!(moves.is_empty());
}

#[test]
fn get_valid_moves_returns_real_legal_moves() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .get_valid_moves(&state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("get_valid_moves should succeed: {error}"));

    assert!(!moves.is_empty());
    assert_eq!(moves[0].piece_id, piece_id(0));
    assert_eq!(moves[0].score_delta, 1);
}

#[test]
fn get_valid_moves_matches_iterator_collection() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let from_iterator = engine
        .valid_moves_iter(&state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
        .collect::<Vec<_>>();

    let from_wrapper = engine
        .get_valid_moves(&state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("get_valid_moves should succeed: {error}"));

    assert_eq!(from_wrapper, from_iterator);
}

#[test]
fn has_any_valid_move_returns_real_boolean() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert_eq!(
        engine.has_any_valid_move(&state, player(1), PlayerColor::Blue),
        Ok(true)
    );
    assert_eq!(
        engine.has_any_valid_move(&state, player(2), PlayerColor::Yellow),
        Ok(false)
    );
}

#[test]
fn score_game_rejects_unfinished_game() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert_eq!(
        engine.score_game(&state, ScoringMode::Basic),
        Err(DomainError::from(RuleViolation::GameNotFinished))
    );
}

#[test]
fn only_unimplemented_methods_return_placeholder_errors() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let command = Command::Pass(PassCommand {
        command_id: command_id(2),
        game_id: state.game_id,
        player_id: player(1),
        color: PlayerColor::Blue,
    });

    assert!(engine.apply(&state, command).is_err());
    assert!(
        engine
            .valid_moves_iter(&state, player(1), PlayerColor::Blue)
            .is_ok()
    );
    assert!(
        engine
            .get_valid_moves(&state, player(1), PlayerColor::Blue)
            .is_ok()
    );
    assert_eq!(
        engine.has_any_valid_move(&state, player(1), PlayerColor::Blue),
        Ok(true)
    );
    assert!(engine.score_game(&state, ScoringMode::Advanced).is_err());
}

#[test]
fn initialized_inventory_accepts_all_current_piece_ids_as_available() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    for inventory in state.inventories {
        for raw_piece_id in 0..21 {
            assert!(inventory.is_available(piece_id(raw_piece_id)));
        }
    }
}

#[test]
fn initialized_state_has_no_piece_usage_or_board_occupancy() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

    assert!(state.board.is_empty());

    for color in PlayerColor::ALL {
        assert_eq!(state.board.occupied(color), BoardMask::EMPTY);
    }

    for inventory in state.inventories {
        assert_eq!(inventory.used_count(), 0);
    }
}
