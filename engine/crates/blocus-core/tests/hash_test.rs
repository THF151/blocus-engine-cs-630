use blocus_core::{
    BlocusEngine, BoardIndex, Command, CommandId, GameConfig, GameId, GameMode, OrientationId,
    PieceId, PlaceCommand, PlayerColor, PlayerId, PlayerSlots, ScoringMode, TurnOrder,
    board_cell_hash, compute_hash_full, inventory_piece_hash,
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

fn config() -> GameConfig {
    let slots = PlayerSlots::two_player(player_id(1), player_id(2))
        .unwrap_or_else(|_| panic!("two-player slots should be valid"));

    GameConfig::try_new(
        game_id(100),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    )
    .unwrap_or_else(|_| panic!("config should be valid"))
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
fn manually_changed_semantic_field_changes_full_hash() {
    let engine = BlocusEngine::new();
    let mut first = engine.initialize_game(config());
    let mut second = first.clone();

    first.version = blocus_core::StateVersion::new(1);
    first.hash = compute_hash_full(&first);

    second.version = blocus_core::StateVersion::new(2);
    second.hash = compute_hash_full(&second);

    assert_ne!(first.hash, second.hash);
}
