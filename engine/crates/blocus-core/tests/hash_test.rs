use blocus_core::{
    BlocusEngine, BoardIndex, BoardMask, Command, CommandId, GameConfig, GameId, GameMode,
    OrientationId, PassCommand, PieceId, PlaceCommand, PlayerColor, PlayerId, PlayerSlots,
    ScoringMode, TurnOrder, board_cell_hash, compute_hash_full, inventory_piece_hash,
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

fn config_with(game_id: GameId, scoring: ScoringMode) -> GameConfig {
    let slots = PlayerSlots::two_player(player_id(1), player_id(2))
        .unwrap_or_else(|_| panic!("two-player slots should be valid"));

    GameConfig::try_new(
        game_id,
        GameMode::TwoPlayer,
        scoring,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    )
    .unwrap_or_else(|_| panic!("config should be valid"))
}

fn config() -> GameConfig {
    config_with(game_id(100), ScoringMode::Basic)
}

#[test]
fn zobrist_constants_are_stable_and_nonzero() {
    assert_ne!(board_cell_hash(PlayerColor::Blue, 0), 0);
    assert_ne!(board_cell_hash(PlayerColor::Yellow, 19), 0);
    assert_ne!(inventory_piece_hash(PlayerColor::Red, 20), 0);

    assert_ne!(
        board_cell_hash(PlayerColor::Blue, 0),
        board_cell_hash(PlayerColor::Blue, 1)
    );
    assert_ne!(
        board_cell_hash(PlayerColor::Blue, 0),
        board_cell_hash(PlayerColor::Yellow, 0)
    );
    assert_ne!(
        inventory_piece_hash(PlayerColor::Blue, 0),
        inventory_piece_hash(PlayerColor::Blue, 1)
    );
}

#[test]
fn initialized_state_hash_matches_full_recomputation() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(config());

    assert_ne!(state.hash.as_u64(), 0);
    assert_eq!(state.hash, compute_hash_full(&state));
}

#[test]
fn equal_semantic_states_have_equal_hashes() {
    let engine = BlocusEngine::new();

    let first = engine.initialize_game(config());
    let second = engine.initialize_game(config());

    assert_eq!(first, second);
    assert_eq!(first.hash, second.hash);
    assert_eq!(compute_hash_full(&first), compute_hash_full(&second));
}

#[test]
fn different_state_after_place_changes_hash_and_matches_recomputation() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(config());

    let command = PlaceCommand {
        command_id: command_id(1),
        game_id: game_id(100),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        anchor: index(0, 0),
    };

    let result = engine
        .apply(&state, Command::Place(command))
        .unwrap_or_else(|error| panic!("opening move should be legal: {error}"));

    assert_ne!(state.hash, result.next_state.hash);
    assert_eq!(
        result.next_state.hash,
        compute_hash_full(&result.next_state)
    );
}

#[test]
fn successful_pass_updates_incremental_hash_to_full_recomputation() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(config());

    state.board.place_mask(
        PlayerColor::Yellow,
        BoardMask::from_index(index(0, 0))
            .union(BoardMask::from_index(index(0, 1)))
            .union(BoardMask::from_index(index(1, 0)))
            .union(BoardMask::from_index(index(1, 1))),
    );
    state.hash = compute_hash_full(&state);

    let result = engine
        .apply(
            &state,
            Command::Pass(PassCommand {
                command_id: command_id(2),
                game_id: game_id(100),
                player_id: player_id(1),
                color: PlayerColor::Blue,
            }),
        )
        .unwrap_or_else(|error| panic!("pass should be legal: {error}"));

    assert_ne!(state.hash, result.next_state.hash);
    assert_eq!(
        result.next_state.hash,
        compute_hash_full(&result.next_state)
    );
}

#[test]
fn replay_identity_fields_do_not_change_position_hash() {
    let engine = BlocusEngine::new();
    let mut first = engine.initialize_game(config_with(game_id(100), ScoringMode::Basic));
    let mut second = engine.initialize_game(config_with(game_id(101), ScoringMode::Basic));

    first.version = blocus_core::StateVersion::new(1);
    first.hash = compute_hash_full(&first);

    second.version = blocus_core::StateVersion::new(2);
    second.hash = compute_hash_full(&second);

    assert_ne!(first.game_id, second.game_id);
    assert_ne!(first.version, second.version);
    assert_eq!(first.hash, second.hash);
}

#[test]
fn rule_context_changes_full_hash() {
    let engine = BlocusEngine::new();
    let basic = engine.initialize_game(config_with(game_id(100), ScoringMode::Basic));
    let advanced = engine.initialize_game(config_with(game_id(100), ScoringMode::Advanced));

    assert_ne!(basic.scoring, advanced.scoring);
    assert_ne!(basic.hash, advanced.hash);
}
