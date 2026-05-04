//! Regression tests for strict Blokus Duo rulebook conformance.
//!
//! These tests are intentionally written against stricter invariants than the
//! current implementation appears to enforce. At the time of writing, many of
//! these tests should fail at runtime until the engine rejects the corresponding
//! Duo misconfigurations and impossible imported states.

use blocus_core::{
    BlocusEngine, BoardIndex, BoardMask, BoardState, DomainError, EngineError, GameConfig, GameId,
    GameMode, GameState, GameStatus, LastPieceByColor, MAX_PLAYER_COLOR_COUNT, PIECE_COUNT,
    PieceId, PieceInventory, PlayerColor, PlayerId, ScoringMode, StateSchemaVersion, StateVersion,
    TurnOrder, TurnState, ZobristHash, compute_hash_full, validate_game_state,
};
use uuid::Uuid;

const GAME_ID_RAW: u128 = 0x900;
const BLACK_PLAYER_RAW: u128 = 0x001;
const WHITE_PLAYER_RAW: u128 = 0x002;

fn game_id() -> GameId {
    GameId::from_uuid(Uuid::from_u128(GAME_ID_RAW))
}

fn black_player() -> PlayerId {
    PlayerId::from_uuid(Uuid::from_u128(BLACK_PLAYER_RAW))
}

fn white_player() -> PlayerId {
    PlayerId::from_uuid(Uuid::from_u128(WHITE_PLAYER_RAW))
}

fn duo_config() -> GameConfig {
    GameConfig::duo(
        game_id(),
        black_player(),
        white_player(),
        PlayerColor::Black,
    )
    .unwrap_or_else(|error| panic!("test fixture Duo config must be valid: {error:?}"))
}

fn initialized_duo_state() -> GameState {
    BlocusEngine::new().initialize_game(duo_config())
}

fn index(row: u8, col: u8) -> BoardIndex {
    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|error| panic!("test coordinate must be on physical board: {error:?}"))
}

fn mask_from_cells(cells: &[(u8, u8)]) -> BoardMask {
    let mut mask = BoardMask::EMPTY;

    for &(row, col) in cells {
        mask.insert(index(row, col));
    }

    mask
}

fn full_inventory() -> PieceInventory {
    PieceInventory::from_used_mask((1u32 << PIECE_COUNT) - 1)
}

fn piece_id(raw_piece_id: u8) -> PieceId {
    PieceId::try_new(raw_piece_id)
        .unwrap_or_else(|error| panic!("test piece id must be valid: {error:?}"))
}

fn inventory_with_piece(raw_piece_id: u8) -> PieceInventory {
    PieceInventory::EMPTY.marked_used(piece_id(raw_piece_id))
}

fn duo_order() -> TurnOrder {
    TurnOrder::duo(PlayerColor::Black)
        .unwrap_or_else(|error| panic!("test fixture Duo turn order must be valid: {error:?}"))
}

fn initial_duo_turn() -> TurnState {
    TurnState::new(duo_order())
}

fn duo_passed_mask() -> u8 {
    PlayerColor::Black.bit() | PlayerColor::White.bit()
}

fn duo_state_from_parts(
    board: &BoardState,
    inventories: [PieceInventory; MAX_PLAYER_COLOR_COUNT],
    last_piece_by_color: LastPieceByColor,
    turn: TurnState,
    status: GameStatus,
) -> GameState {
    let config = duo_config();

    let mut state = GameState {
        schema_version: StateSchemaVersion::CURRENT,
        game_id: config.game_id(),
        mode: GameMode::Duo,
        scoring: ScoringMode::Advanced,
        turn_order: config.turn_order(),
        player_slots: config.player_slots(),
        board: *board,
        inventories,
        last_piece_by_color,
        turn,
        status,
        version: StateVersion::INITIAL,
        hash: ZobristHash::ZERO,
    };

    state.hash = compute_hash_full(&state);
    state
}

fn empty_duo_inventories() -> [PieceInventory; MAX_PLAYER_COLOR_COUNT] {
    [PieceInventory::EMPTY; MAX_PLAYER_COLOR_COUNT]
}

fn assert_corrupted_state(result: &Result<(), DomainError>) {
    assert!(
        matches!(
            result,
            Err(DomainError::EngineError(EngineError::CorruptedState))
        ),
        "expected EngineError::CorruptedState, got {result:?}"
    );
}

#[test]
fn duo_validate_state_accepts_two_monomino_opening_starts() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);
    board_masks[PlayerColor::White.index()] = mask_from_cells(&[(9, 9)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);
    inventories[PlayerColor::White.index()] = inventory_with_piece(0);

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));
    last_piece_by_color.set(PlayerColor::White, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        TurnState::from_parts(PlayerColor::Black, 0, 0),
        GameStatus::InProgress,
    );

    assert_eq!(validate_game_state(&state), Ok(()));
}

#[test]
fn duo_scoring_must_reject_basic_scoring_even_if_caller_requests_it() {
    let engine = BlocusEngine::new();
    let mut state = initialized_duo_state();
    state.status = GameStatus::Finished;
    state.hash = compute_hash_full(&state);

    let result = engine.score_game(&state, ScoringMode::Basic);

    assert!(
        matches!(
            result,
            Err(DomainError::InputError(_) | DomainError::EngineError(_))
        ),
        "Duo rulebook scoring is advanced-only; score_game must reject BASIC for Duo, got {result:?}"
    );
}

#[test]
fn duo_scoring_must_reject_scoring_argument_that_differs_from_state_scoring() {
    let engine = BlocusEngine::new();
    let mut state = initialized_duo_state();
    state.status = GameStatus::Finished;
    state.hash = compute_hash_full(&state);

    let result = engine.score_game(&state, ScoringMode::Basic);

    assert!(
        result.is_err(),
        "score_game must not allow callers to override state.scoring for a finished Duo state"
    );
}

#[test]
fn duo_validate_state_rejects_black_occupied_cells_with_empty_inventory() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        empty_duo_inventories(),
        LastPieceByColor::EMPTY,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_white_occupied_cells_with_empty_inventory() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::White.index()] = mask_from_cells(&[(9, 9)]);

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        empty_duo_inventories(),
        LastPieceByColor::EMPTY,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_used_inventory_with_no_board_cells() {
    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);

    let state = duo_state_from_parts(
        &BoardState::EMPTY,
        inventories,
        LastPieceByColor::EMPTY,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_inventory_square_count_smaller_than_board_occupancy() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4), (4, 5)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        LastPieceByColor::EMPTY,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_inventory_square_count_larger_than_board_occupancy() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(1);

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        LastPieceByColor::EMPTY,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_last_piece_for_color_with_empty_inventory() {
    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::EMPTY,
        empty_duo_inventories(),
        last_piece_by_color,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_last_piece_not_marked_used_in_inventory() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(1));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_used_inventory_without_last_piece() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        LastPieceByColor::EMPTY,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_first_black_piece_that_does_not_cover_duo_starting_point() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(0, 0)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        initial_duo_turn(),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_first_white_piece_that_does_not_cover_remaining_starting_point() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);
    board_masks[PlayerColor::White.index()] = mask_from_cells(&[(0, 0)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);
    inventories[PlayerColor::White.index()] = inventory_with_piece(0);

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));
    last_piece_by_color.set(PlayerColor::White, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        TurnState::from_parts(PlayerColor::Black, 0, 0),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_both_players_opening_on_same_starting_point_shape_history_impossible()
{
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);
    board_masks[PlayerColor::White.index()] = mask_from_cells(&[(4, 5)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);
    inventories[PlayerColor::White.index()] = inventory_with_piece(0);

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));
    last_piece_by_color.set(PlayerColor::White, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        TurnState::from_parts(PlayerColor::Black, 0, 0),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_finished_state_with_all_pieces_used_but_too_few_board_squares() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = full_inventory();

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        TurnState::from_parts(PlayerColor::Black, duo_passed_mask(), 0),
        GameStatus::Finished,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_passed_finished_state_with_no_board_inventory_consistency() {
    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);
    inventories[PlayerColor::White.index()] = inventory_with_piece(0);

    let state = duo_state_from_parts(
        &BoardState::EMPTY,
        inventories,
        LastPieceByColor::EMPTY,
        TurnState::from_parts(PlayerColor::Black, duo_passed_mask(), 0),
        GameStatus::Finished,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_black_and_white_inventory_with_only_one_start_covered() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4)]);
    board_masks[PlayerColor::White.index()] = mask_from_cells(&[(4, 5)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(0);
    inventories[PlayerColor::White.index()] = inventory_with_piece(0);

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));
    last_piece_by_color.set(PlayerColor::White, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        TurnState::from_parts(PlayerColor::Black, 0, 0),
        GameStatus::InProgress,
    );

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_validate_state_rejects_non_duo_turn_order_even_if_current_color_is_active() {
    let config = duo_config();

    let mut state = GameState {
        schema_version: StateSchemaVersion::CURRENT,
        game_id: config.game_id(),
        mode: GameMode::Duo,
        scoring: ScoringMode::Advanced,
        turn_order: TurnOrder::OFFICIAL_FIXED,
        player_slots: config.player_slots(),
        board: BoardState::EMPTY,
        inventories: empty_duo_inventories(),
        last_piece_by_color: LastPieceByColor::EMPTY,
        turn: TurnState::from_parts(PlayerColor::Black, 0, 0),
        status: GameStatus::InProgress,
        version: StateVersion::INITIAL,
        hash: ZobristHash::ZERO,
    };

    state.hash = compute_hash_full(&state);

    assert_corrupted_state(&validate_game_state(&state));
}

#[test]
fn duo_player_slots_rejects_same_player_controlling_both_colors() {
    let same_player_slots = blocus_core::PlayerSlots::duo(black_player(), black_player());

    assert!(
        same_player_slots.is_err(),
        "Duo setup must reject same player controlling black and white"
    );
}

#[test]
fn duo_validate_state_rejects_finished_state_with_forged_monomino_last_bonus() {
    let mut board_masks = [BoardMask::EMPTY; MAX_PLAYER_COLOR_COUNT];
    board_masks[PlayerColor::Black.index()] = mask_from_cells(&[(4, 4), (4, 5)]);

    let mut inventories = empty_duo_inventories();
    inventories[PlayerColor::Black.index()] = inventory_with_piece(1);

    let mut last_piece_by_color = LastPieceByColor::EMPTY;
    last_piece_by_color.set(PlayerColor::Black, piece_id(0));

    let state = duo_state_from_parts(
        &BoardState::from_occupied_by_color(board_masks),
        inventories,
        last_piece_by_color,
        TurnState::from_parts(PlayerColor::Black, duo_passed_mask(), 0),
        GameStatus::Finished,
    );

    assert_corrupted_state(&validate_game_state(&state));
}
