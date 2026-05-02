use blocus_core::{
    BOARD_SIZE, BlocusEngine, BoardIndex, Command, CommandId, GameConfig, GameId, GameMode,
    GameState, LegalMove, OrientationId, PIECE_COUNT, PieceId, PlaceCommand, PlayerColor, PlayerId,
    PlayerSlots, ScoringMode, StateVersion, TurnOrder, build_placement, standard_repository,
    validate_place_command,
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

fn opening_place_command(
    color: PlayerColor,
    player_id: PlayerId,
    piece_id: PieceId,
    orientation_id: OrientationId,
    anchor: BoardIndex,
) -> PlaceCommand {
    PlaceCommand {
        command_id: command_id(1),
        game_id: game_id(100),
        player_id,
        color,
        piece_id,
        orientation_id,
        anchor,
    }
}

fn assert_sorted_by_piece_orientation_anchor(moves: &[LegalMove]) {
    for pair in moves.windows(2) {
        let left = pair[0];
        let right = pair[1];
        let left_key = (
            left.piece_id.as_u8(),
            left.orientation_id.as_u8(),
            left.anchor.bit_index(),
        );
        let right_key = (
            right.piece_id.as_u8(),
            right.orientation_id.as_u8(),
            right.anchor.bit_index(),
        );
        assert!(
            left_key < right_key,
            "moves should be strictly sorted and duplicate-free: left={left_key:?}, right={right_key:?}"
        );
    }
}

fn brute_force_moves(state: &GameState, player_id: PlayerId, color: PlayerColor) -> Vec<LegalMove> {
    let repository = standard_repository();
    let mut moves = Vec::new();

    for raw_piece_id in 0..PIECE_COUNT {
        let piece_id = piece_id(raw_piece_id);

        if state.inventories[color.index()].is_used(piece_id) {
            continue;
        }

        let piece = repository.piece(piece_id);

        for raw_orientation_id in 0..piece.orientation_count() {
            let orientation_id = orientation_id(raw_orientation_id);

            for row in 0..BOARD_SIZE {
                for col in 0..BOARD_SIZE {
                    let command = PlaceCommand {
                        command_id: command_id(9_999),
                        game_id: state.game_id,
                        player_id,
                        color,
                        piece_id,
                        orientation_id,
                        anchor: index(row, col),
                    };

                    if let Ok(placement) = validate_place_command(state, command, repository) {
                        moves.push(LegalMove {
                            piece_id,
                            orientation_id,
                            anchor: placement.anchor(),
                            score_delta: placement.square_count(),
                        });
                    }
                }
            }
        }
    }

    moves
}

fn state_after_four_opening_moves(engine: BlocusEngine) -> GameState {
    let state = engine.initialize_game(two_player_config());
    let state = engine
        .apply(
            &state,
            Command::Place(opening_place_command(
                PlayerColor::Blue,
                player_id(1),
                piece_id(0),
                orientation_id(0),
                index(0, 0),
            )),
        )
        .unwrap_or_else(|error| panic!("blue opening move should apply: {error}"))
        .next_state;
    let state = engine
        .apply(
            &state,
            Command::Place(opening_place_command(
                PlayerColor::Yellow,
                player_id(2),
                piece_id(0),
                orientation_id(0),
                index(0, 19),
            )),
        )
        .unwrap_or_else(|error| panic!("yellow opening move should apply: {error}"))
        .next_state;
    let state = engine
        .apply(
            &state,
            Command::Place(opening_place_command(
                PlayerColor::Red,
                player_id(1),
                piece_id(0),
                orientation_id(0),
                index(19, 19),
            )),
        )
        .unwrap_or_else(|error| panic!("red opening move should apply: {error}"))
        .next_state;

    engine
        .apply(
            &state,
            Command::Place(opening_place_command(
                PlayerColor::Green,
                player_id(2),
                piece_id(0),
                orientation_id(0),
                index(19, 0),
            )),
        )
        .unwrap_or_else(|error| panic!("green opening move should apply: {error}"))
        .next_state
}

#[test]
fn legal_move_iter_yields_deterministic_opening_moves_for_current_color() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .valid_moves_iter(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .take(8)
        .collect::<Vec<_>>();

    assert_eq!(
        moves[0],
        LegalMove {
            piece_id: piece_id(0),
            orientation_id: orientation_id(0),
            anchor: index(0, 0),
            score_delta: 1,
        }
    );
    assert_sorted_by_piece_orientation_anchor(&moves);
}

#[test]
fn legal_move_iter_is_lazy_and_reusable_by_reconstruction() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let first = engine
        .valid_moves_iter(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .take(12)
        .collect::<Vec<_>>();

    let second = engine
        .valid_moves_iter(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .take(12)
        .collect::<Vec<_>>();

    assert_eq!(first, second);
    assert_sorted_by_piece_orientation_anchor(&first);
}

#[test]
fn legal_move_iter_excludes_wrong_turn_moves() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .valid_moves_iter(&state, player_id(2), PlayerColor::Yellow)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .collect::<Vec<_>>();

    assert!(moves.is_empty());
}

#[test]
fn legal_move_iter_excludes_uncontrolled_color_moves() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .valid_moves_iter(&state, player_id(2), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .collect::<Vec<_>>();

    assert!(moves.is_empty());
}

#[test]
fn legal_move_iter_excludes_already_used_piece() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(two_player_config());

    // Mark I1 as used in Blue's inventory while Blue is still the active
    // color. The iterator's piece-availability filter must skip any piece
    // that is marked used in the iterating color's own inventory.
    state.inventories[PlayerColor::Blue.index()].mark_used(piece_id(0));

    let moves = engine
        .valid_moves_iter(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .collect::<Vec<_>>();

    assert!(
        moves
            .iter()
            .all(|legal_move| legal_move.piece_id != piece_id(0)),
        "Blue's iterator must not yield a piece that is marked used in Blue's own inventory"
    );
    assert!(
        !moves.is_empty(),
        "Blue should still have legal opening moves with the remaining pieces"
    );
}

#[test]
fn applied_used_piece_is_recorded_in_correct_color_inventory() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let command = opening_place_command(
        PlayerColor::Blue,
        player_id(1),
        piece_id(0),
        orientation_id(0),
        index(0, 0),
    );
    let result = engine
        .apply(&state, Command::Place(command))
        .unwrap_or_else(|error| panic!("opening move should be legal: {error}"));

    // Inventories are per-color: Blue consumed I1, Yellow has not.
    assert!(result.next_state.inventories[PlayerColor::Blue.index()].is_used(piece_id(0)));
    assert!(!result.next_state.inventories[PlayerColor::Yellow.index()].is_used(piece_id(0)));
}

#[test]
fn legal_move_iter_yields_duplicate_free_output() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let moves = engine
        .valid_moves_iter(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .collect::<Vec<_>>();

    let unique = moves.iter().copied().collect::<HashSet<_>>();
    assert_eq!(moves.len(), unique.len());
    assert_sorted_by_piece_orientation_anchor(&moves);
}

#[test]
fn get_valid_moves_collects_the_same_moves_as_the_iterator() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let from_iterator = engine
        .valid_moves_iter(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("iterator construction should succeed: {error}"))
        .collect::<Vec<_>>();

    let from_wrapper = engine
        .get_valid_moves(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("get_valid_moves should succeed: {error}"));

    assert_eq!(from_wrapper, from_iterator);
    assert!(!from_wrapper.is_empty());
}

#[test]
fn optimized_move_generation_matches_brute_force_for_opening() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let optimized = engine
        .get_valid_moves(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("optimized move generation should succeed: {error}"));
    let brute_force = brute_force_moves(&state, player_id(1), PlayerColor::Blue);

    assert_eq!(optimized, brute_force);
    assert_sorted_by_piece_orientation_anchor(&optimized);
}

#[test]
fn optimized_move_generation_matches_brute_force_after_frontier_exists() {
    let engine = BlocusEngine::new();
    let state = state_after_four_opening_moves(engine);

    let optimized = engine
        .get_valid_moves(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("optimized move generation should succeed: {error}"));
    let brute_force = brute_force_moves(&state, player_id(1), PlayerColor::Blue);

    assert_eq!(optimized, brute_force);
    assert_sorted_by_piece_orientation_anchor(&optimized);
}

#[test]
fn generated_frontier_moves_do_not_overlap_or_touch_edges() {
    let engine = BlocusEngine::new();
    let state = state_after_four_opening_moves(engine);
    let own = state.board.occupied(PlayerColor::Blue);
    let occupied = state.board.occupied_all();
    let frontier = own.diagonal_frontier();
    let edge_forbidden = own.edge_neighbors();

    let moves = engine
        .get_valid_moves(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("move generation should succeed: {error}"));

    assert!(!moves.is_empty());

    for legal_move in moves {
        let piece = standard_repository().piece(legal_move.piece_id);
        let orientation = piece
            .orientation(legal_move.orientation_id)
            .unwrap_or_else(|| panic!("generated orientation should exist"));
        let placement = build_placement(legal_move.piece_id, orientation, legal_move.anchor)
            .unwrap_or_else(|error| panic!("generated placement should fit: {error}"));

        assert!(placement.mask().intersects(frontier));
        assert!(!placement.mask().intersects(occupied));
        assert!(!placement.mask().intersects(edge_forbidden));
    }
}

#[test]
fn has_any_valid_move_returns_true_for_opening_current_color() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert_eq!(
        engine.has_any_valid_move(&state, player_id(1), PlayerColor::Blue),
        Ok(true)
    );
}

#[test]
fn has_any_valid_move_returns_false_for_wrong_turn() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert_eq!(
        engine.has_any_valid_move(&state, player_id(2), PlayerColor::Yellow),
        Ok(false)
    );
}

#[test]
fn has_any_valid_move_returns_false_for_uncontrolled_color() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert_eq!(
        engine.has_any_valid_move(&state, player_id(2), PlayerColor::Blue),
        Ok(false)
    );
}

#[test]
fn move_generation_does_not_mutate_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());
    let original = state.clone();

    let _moves = engine
        .get_valid_moves(&state, player_id(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("get_valid_moves should succeed: {error}"));

    assert_eq!(state, original);
    assert_eq!(state.version, StateVersion::INITIAL);
    assert!(state.board.is_empty());
}
