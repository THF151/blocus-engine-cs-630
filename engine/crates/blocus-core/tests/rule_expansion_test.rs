use blocus_core::{
    BlocusEngine, BoardIndex, BoardMask, Command, CommandId, DomainError, GameConfig, GameId,
    GameMode, PassCommand, PieceId, PlaceCommand, PlayerColor, PlayerId, PlayerSlots,
    RuleViolation, ScoringMode, TurnOrder,
};
use uuid::Uuid;

#[derive(Clone, Copy)]
struct PlaceSpec {
    command_value: u128,
    game_id: Uuid,
    player_id: PlayerId,
    color: PlayerColor,
    piece_id: PieceId,
    orientation_id: blocus_core::OrientationId,
    row: u8,
    col: u8,
}

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

fn board_index(row: u8, col: u8) -> BoardIndex {
    BoardIndex::from_row_col(row, col)
        .unwrap_or_else(|error| panic!("board index ({row}, {col}) should be valid: {error}"))
}

fn two_player_slots() -> PlayerSlots {
    let Ok(slots) = PlayerSlots::two_player(player(1), player(2)) else {
        panic!("two-player slots should be valid");
    };

    slots
}

fn two_player_config() -> GameConfig {
    let Ok(config) = GameConfig::try_new(
        GameId::from_uuid(uuid(100)),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        two_player_slots(),
    ) else {
        panic!("two-player config should be valid");
    };

    config
}

fn place_command(spec: PlaceSpec) -> Command {
    Command::Place(PlaceCommand {
        command_id: command_id(spec.command_value),
        game_id: GameId::from_uuid(spec.game_id),
        player_id: spec.player_id,
        color: spec.color,
        piece_id: spec.piece_id,
        orientation_id: spec.orientation_id,
        anchor: board_index(spec.row, spec.col),
    })
}

fn pass_command(
    command_value: u128,
    game_id: GameId,
    player_id: PlayerId,
    color: PlayerColor,
) -> Command {
    Command::Pass(PassCommand {
        command_id: command_id(command_value),
        game_id,
        player_id,
        color,
    })
}

fn command_from_legal_move(
    state_game_id: GameId,
    player_id: PlayerId,
    color: PlayerColor,
    command_value: u128,
    legal_move: blocus_core::LegalMove,
) -> Command {
    Command::Place(PlaceCommand {
        command_id: command_id(command_value),
        game_id: state_game_id,
        player_id,
        color,
        piece_id: legal_move.piece_id,
        orientation_id: legal_move.orientation_id,
        anchor: legal_move.anchor,
    })
}

fn apply_first_legal_move(
    engine: BlocusEngine,
    state: &blocus_core::GameState,
    player_id: PlayerId,
    color: PlayerColor,
    command_value: u128,
) -> blocus_core::GameState {
    let legal_move = engine
        .get_valid_moves(state, player_id, color)
        .unwrap_or_else(|error| panic!("valid moves should generate: {error}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("expected at least one legal move for {color:?}"));

    engine
        .apply(
            state,
            command_from_legal_move(state.game_id, player_id, color, command_value, legal_move),
        )
        .unwrap_or_else(|error| panic!("legal move should apply: {error}"))
        .next_state
}

fn assert_rejected_without_mutation(
    result: Result<blocus_core::GameResult, DomainError>,
    original: &blocus_core::GameState,
    after_attempt: &blocus_core::GameState,
) -> DomainError {
    assert_eq!(after_attempt, original);

    match result {
        Ok(_) => panic!("command should have been rejected"),
        Err(error) => error,
    }
}

#[test]
fn initialized_state_hash_matches_full_recomputation() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert_eq!(state.hash, blocus_core::compute_hash_full(&state));
}

#[test]
fn repeated_initialization_is_semantically_stable() {
    let engine = BlocusEngine::new();

    let first = engine.initialize_game(two_player_config());
    let second = engine.initialize_game(two_player_config());

    assert_eq!(first, second);
    assert_eq!(first.hash, second.hash);
    assert_eq!(first.hash, blocus_core::compute_hash_full(&first));
}

#[test]
fn different_game_ids_produce_different_hashes() {
    let engine = BlocusEngine::new();

    let first = engine.initialize_game(two_player_config());

    let Ok(config) = GameConfig::try_new(
        GameId::from_uuid(uuid(101)),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        two_player_slots(),
    ) else {
        panic!("config should be valid");
    };

    let second = engine.initialize_game(config);

    assert_ne!(first.game_id, second.game_id);
    assert_ne!(first.hash, second.hash);
}

#[test]
fn opening_move_generation_is_empty_for_non_current_color() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .get_valid_moves(&state, player(2), PlayerColor::Yellow)
        .unwrap_or_else(|error| panic!("move generation should succeed: {error}"));

    assert!(moves.is_empty());
}

#[test]
fn opening_move_generation_contains_only_corner_covering_moves_for_current_color() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .get_valid_moves(&state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("move generation should succeed: {error}"));

    assert!(!moves.is_empty());

    for (offset, legal_move) in moves.into_iter().take(16).enumerate() {
        let Ok(offset) = u128::try_from(offset) else {
            panic!("offset should fit into u128");
        };

        let next_state = engine
            .apply(
                &state,
                command_from_legal_move(
                    state.game_id,
                    player(1),
                    PlayerColor::Blue,
                    1_000 + offset,
                    legal_move,
                ),
            )
            .unwrap_or_else(|error| panic!("generated move should apply: {error}"))
            .next_state;

        assert!(
            next_state
                .board
                .occupied(PlayerColor::Blue)
                .contains(board_index(0, 0))
        );
    }
}

#[test]
fn legal_move_candidates_apply_successfully_without_mutating_original_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let legal_moves = engine
        .get_valid_moves(&state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("move generation should succeed: {error}"));

    assert!(!legal_moves.is_empty());

    for (offset, legal_move) in legal_moves.into_iter().take(8).enumerate() {
        let Ok(offset) = u128::try_from(offset) else {
            panic!("offset should fit into u128");
        };

        let result = engine
            .apply(
                &state,
                command_from_legal_move(
                    state.game_id,
                    player(1),
                    PlayerColor::Blue,
                    2_000 + offset,
                    legal_move,
                ),
            )
            .unwrap_or_else(|error| panic!("generated legal move should apply: {error}"));

        assert_eq!(state, original);
        assert_eq!(state.hash, blocus_core::compute_hash_full(&state));
        assert_eq!(
            result.next_state.hash,
            blocus_core::compute_hash_full(&result.next_state)
        );
    }
}

#[test]
fn legal_move_generation_excludes_used_piece_after_successful_move() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let legal_move = engine
        .get_valid_moves(&state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("move generation should succeed: {error}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("expected opening legal move"));

    let used_piece = legal_move.piece_id;

    let next_state = engine
        .apply(
            &state,
            command_from_legal_move(
                state.game_id,
                player(1),
                PlayerColor::Blue,
                3_000,
                legal_move,
            ),
        )
        .unwrap_or_else(|error| panic!("opening move should apply: {error}"))
        .next_state;

    let blue_moves = engine
        .get_valid_moves(&next_state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("post-move generation should succeed: {error}"));

    assert!(
        blue_moves
            .iter()
            .all(|candidate| candidate.piece_id != used_piece)
    );
}

#[test]
fn successful_place_updates_hash_to_full_recomputation() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let next_state = apply_first_legal_move(engine, &state, player(1), PlayerColor::Blue, 4_000);

    assert_ne!(next_state.hash, state.hash);
    assert_eq!(next_state.hash, blocus_core::compute_hash_full(&next_state));
}

#[test]
fn invalid_place_does_not_mutate_original_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let command = place_command(PlaceSpec {
        command_value: 5_000,
        game_id: uuid(100),
        player_id: player(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        row: 1,
        col: 1,
    });

    let result = engine.apply(&state, command);
    let error = assert_rejected_without_mutation(result, &original, &state);

    assert_eq!(error.category(), "rule_violation");
    assert_eq!(state.hash, blocus_core::compute_hash_full(&state));
}

#[test]
fn place_rejects_wrong_turn_without_mutating_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let command = place_command(PlaceSpec {
        command_value: 6_000,
        game_id: uuid(100),
        player_id: player(2),
        color: PlayerColor::Yellow,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        row: 0,
        col: 19,
    });

    let result = engine.apply(&state, command);
    let error = assert_rejected_without_mutation(result, &original, &state);

    assert_eq!(error.category(), "rule_violation");
}

#[test]
fn place_rejects_wrong_game_id_without_mutating_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let command = place_command(PlaceSpec {
        command_value: 7_000,
        game_id: uuid(999),
        player_id: player(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        row: 0,
        col: 0,
    });

    let result = engine.apply(&state, command);
    let error = assert_rejected_without_mutation(result, &original, &state);

    assert_eq!(error.category(), "input_error");
}

fn orientation_id(value: u8) -> blocus_core::OrientationId {
    let Ok(orientation_id) = blocus_core::OrientationId::try_new(value) else {
        panic!("orientation id {value} should be valid");
    };

    orientation_id
}

#[test]
fn place_rejects_uncontrolled_color_without_mutating_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let command = place_command(PlaceSpec {
        command_value: 8_000,
        game_id: uuid(100),
        player_id: player(2),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        row: 0,
        col: 0,
    });

    let result = engine.apply(&state, command);
    let error = assert_rejected_without_mutation(result, &original, &state);

    assert_eq!(error.category(), "rule_violation");
}

#[test]
fn pass_rejects_while_move_exists_without_mutating_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let command = pass_command(9_000, state.game_id, player(1), PlayerColor::Blue);

    let result = engine.apply(&state, command);

    assert_eq!(
        result,
        Err(DomainError::from(
            RuleViolation::PassNotAllowedBecauseMoveExists
        ))
    );
    assert_eq!(state, original);
    assert_eq!(state.hash, blocus_core::compute_hash_full(&state));
}

#[test]
fn pass_rejects_wrong_turn_without_mutating_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let command = pass_command(10_000, state.game_id, player(2), PlayerColor::Yellow);

    let result = engine.apply(&state, command);
    let error = assert_rejected_without_mutation(result, &original, &state);

    assert_eq!(error.category(), "rule_violation");
}

#[test]
fn pass_rejects_wrong_game_id_without_mutating_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let command = pass_command(
        11_000,
        GameId::from_uuid(uuid(999)),
        player(1),
        PlayerColor::Blue,
    );

    let result = engine.apply(&state, command);
    let error = assert_rejected_without_mutation(result, &original, &state);

    assert_eq!(error.category(), "input_error");
}

#[test]
fn board_masks_do_not_overlap_after_successful_opening_sequence() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(two_player_config());

    state = apply_first_legal_move(engine, &state, player(1), PlayerColor::Blue, 12_000);
    state = apply_first_legal_move(engine, &state, player(2), PlayerColor::Yellow, 12_001);
    state = apply_first_legal_move(engine, &state, player(1), PlayerColor::Red, 12_002);
    state = apply_first_legal_move(engine, &state, player(2), PlayerColor::Green, 12_003);

    let mut occupied = BoardMask::EMPTY;

    for color in PlayerColor::ALL {
        let color_mask = state.board.occupied(color);
        assert!(!occupied.intersects(color_mask));
        occupied = occupied.union(color_mask);
    }

    assert_eq!(state.hash, blocus_core::compute_hash_full(&state));
}
