use blocus_core::{
    ALL_PIECES_MASK, BlocusEngine, BoardGeometry, BoardIndex, BoardMask, Command, CommandId,
    DomainError, EngineError, GameConfig, GameId, GameMode, GameState, InputError, OrientationId,
    PieceId, PlaceCommand, PlayerColor, PlayerId, RuleViolation, ScoringMode, TurnOrder,
    compute_hash_full, standard_repository, validate_game_state, validate_place_command,
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

fn duo_config() -> GameConfig {
    GameConfig::duo(game_id(900), player_id(1), player_id(2), PlayerColor::Black)
        .unwrap_or_else(|error| panic!("Duo config should be valid: {error}"))
}

fn place_command(
    color: PlayerColor,
    player_id: PlayerId,
    piece_id: PieceId,
    orientation_id: OrientationId,
    anchor: BoardIndex,
) -> PlaceCommand {
    PlaceCommand {
        command_id: command_id(1),
        game_id: game_id(900),
        player_id,
        color,
        piece_id,
        orientation_id,
        anchor,
    }
}

fn apply_monomino(
    engine: BlocusEngine,
    state: &GameState,
    color: PlayerColor,
    player: PlayerId,
    anchor: BoardIndex,
) -> GameState {
    engine
        .apply(
            state,
            Command::Place(place_command(
                color,
                player,
                piece_id(0),
                orientation_id(0),
                anchor,
            )),
        )
        .unwrap_or_else(|error| panic!("Duo monomino move should apply: {error}"))
        .next_state
}

fn first_duo_cells(count: u32) -> BoardMask {
    let mut mask = BoardMask::EMPTY;
    let mut remaining = count;

    for row in 0..14 {
        for col in 0..14 {
            if remaining == 0 {
                return mask;
            }

            mask.insert(index(row, col));
            remaining -= 1;
        }
    }

    mask
}

#[test]
fn duo_geometry_uses_fourteen_by_fourteen_playable_area() {
    assert_eq!(BoardGeometry::classic().playable_mask().count(), 400);
    assert_eq!(BoardGeometry::duo().size(), 14);
    assert_eq!(BoardGeometry::duo().playable_mask().count(), 196);

    assert!(!BoardGeometry::duo().playable_mask().contains(index(14, 0)));
    assert!(!BoardGeometry::duo().playable_mask().contains(index(0, 14)));
    assert!(!BoardGeometry::duo().playable_mask().contains(index(19, 19)));
}

#[test]
fn duo_config_initializes_black_white_turn_order_and_advanced_scoring() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(duo_config());

    assert_eq!(state.mode, GameMode::Duo);
    assert_eq!(state.scoring, ScoringMode::Advanced);
    assert_eq!(state.turn.current_color(), PlayerColor::Black);
    assert_eq!(
        state.turn_order.colors(),
        [PlayerColor::Black, PlayerColor::White]
    );
    assert_eq!(
        state.mode.active_colors(),
        [PlayerColor::Black, PlayerColor::White]
    );
}

#[test]
fn duo_rejects_basic_scoring_and_classic_turn_order() {
    let slots = blocus_core::PlayerSlots::duo(player_id(1), player_id(2))
        .unwrap_or_else(|error| panic!("Duo slots should be valid: {error}"));

    assert_eq!(
        GameConfig::try_new(
            game_id(900),
            GameMode::Duo,
            ScoringMode::Basic,
            TurnOrder::duo(PlayerColor::Black)
                .unwrap_or_else(|error| panic!("black-first Duo turn order is valid: {error}")),
            slots,
        ),
        Err(InputError::InvalidGameConfig)
    );
    assert_eq!(
        GameConfig::try_new(
            game_id(900),
            GameMode::Duo,
            ScoringMode::Advanced,
            TurnOrder::OFFICIAL_FIXED,
            slots,
        ),
        Err(InputError::InvalidGameConfig)
    );
}

#[test]
fn duo_opening_rules_use_two_shared_starting_points() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(duo_config());

    assert_eq!(
        engine.apply(
            &state,
            Command::Place(place_command(
                PlayerColor::Black,
                player_id(1),
                piece_id(0),
                orientation_id(0),
                index(0, 0),
            )),
        ),
        Err(DomainError::from(RuleViolation::MissingCornerContact))
    );
    assert_eq!(
        engine.apply(
            &state,
            Command::Place(place_command(
                PlayerColor::Black,
                player_id(1),
                piece_id(0),
                orientation_id(0),
                index(14, 14),
            )),
        ),
        Err(DomainError::from(RuleViolation::OutOfBounds))
    );

    let state = apply_monomino(
        engine,
        &state,
        PlayerColor::Black,
        player_id(1),
        index(4, 4),
    );

    assert_eq!(
        engine.apply(
            &state,
            Command::Place(place_command(
                PlayerColor::White,
                player_id(2),
                piece_id(0),
                orientation_id(0),
                index(4, 4),
            )),
        ),
        Err(DomainError::from(RuleViolation::Overlap))
    );
    assert_eq!(
        engine.apply(
            &state,
            Command::Place(place_command(
                PlayerColor::White,
                player_id(2),
                piece_id(0),
                orientation_id(0),
                index(0, 0),
            )),
        ),
        Err(DomainError::from(RuleViolation::MissingCornerContact))
    );

    let state = apply_monomino(
        engine,
        &state,
        PlayerColor::White,
        player_id(2),
        index(9, 9),
    );
    assert!(
        state
            .board
            .occupied(PlayerColor::White)
            .contains(index(9, 9))
    );
}

#[test]
fn duo_initial_movegen_covers_starts_and_never_leaves_fourteen_by_fourteen_board() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(duo_config());
    let repository = standard_repository();
    let starts = BoardMask::from_index(index(4, 4)).union(BoardMask::from_index(index(9, 9)));

    let moves = engine
        .get_valid_moves(&state, player_id(1), PlayerColor::Black)
        .unwrap_or_else(|error| panic!("Duo initial movegen should succeed: {error}"));

    assert!(!moves.is_empty());

    for legal_move in moves {
        let command = place_command(
            PlayerColor::Black,
            player_id(1),
            legal_move.piece_id,
            legal_move.orientation_id,
            legal_move.anchor,
        );
        let placement = validate_place_command(&state, command, repository)
            .unwrap_or_else(|error| panic!("generated move should validate: {error}"));

        assert!(placement.mask().intersects(starts));

        for cell in placement.mask().indices() {
            assert!(cell.row() < 14);
            assert!(cell.col() < 14);
        }
    }
}

#[test]
fn duo_second_player_movegen_uses_only_remaining_start() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(duo_config());
    let state = apply_monomino(
        engine,
        &state,
        PlayerColor::Black,
        player_id(1),
        index(4, 4),
    );
    let repository = standard_repository();
    let first_start = BoardMask::from_index(index(4, 4));
    let remaining_start = BoardMask::from_index(index(9, 9));

    let moves = engine
        .get_valid_moves(&state, player_id(2), PlayerColor::White)
        .unwrap_or_else(|error| panic!("White opening movegen should succeed: {error}"));

    assert!(!moves.is_empty());

    for legal_move in moves {
        let command = place_command(
            PlayerColor::White,
            player_id(2),
            legal_move.piece_id,
            legal_move.orientation_id,
            legal_move.anchor,
        );
        let placement = validate_place_command(&state, command, repository)
            .unwrap_or_else(|error| panic!("generated move should validate: {error}"));

        assert!(placement.mask().intersects(remaining_start));
        assert!(!placement.mask().intersects(first_start));
    }
}

#[test]
fn duo_contact_rules_match_classic_after_opening() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(duo_config());
    let state = apply_monomino(
        engine,
        &state,
        PlayerColor::Black,
        player_id(1),
        index(4, 4),
    );
    let state = apply_monomino(
        engine,
        &state,
        PlayerColor::White,
        player_id(2),
        index(9, 9),
    );

    let legal = engine
        .get_valid_moves(&state, player_id(1), PlayerColor::Black)
        .unwrap_or_else(|error| panic!("Black follow-up movegen should succeed: {error}"));
    assert!(!legal.is_empty());

    let edge_contact_error = (0..blocus_core::PIECE_COUNT)
        .flat_map(|raw_piece| {
            let piece_id = piece_id(raw_piece);
            let piece = standard_repository().piece(piece_id);
            (0..piece.orientation_count()).map(move |raw_orientation| (piece_id, raw_orientation))
        })
        .flat_map(|(piece_id, raw_orientation)| {
            (0..14).flat_map(move |row| {
                (0..14).map(move |col| (piece_id, orientation_id(raw_orientation), row, col))
            })
        })
        .find_map(|(piece_id, orientation_id, row, col)| {
            let command = place_command(
                PlayerColor::Black,
                player_id(1),
                piece_id,
                orientation_id,
                index(row, col),
            );

            match validate_place_command(&state, command, standard_repository()) {
                Err(error) if error == DomainError::from(RuleViolation::IllegalEdgeContact) => {
                    Some(error)
                }
                _ => None,
            }
        });

    assert_eq!(
        edge_contact_error,
        Some(DomainError::from(RuleViolation::IllegalEdgeContact))
    );

    let first_legal = legal[0];
    let result = engine.apply(
        &state,
        Command::Place(place_command(
            PlayerColor::Black,
            player_id(1),
            first_legal.piece_id,
            first_legal.orientation_id,
            first_legal.anchor,
        )),
    );

    assert!(result.is_ok());
}

#[test]
fn duo_rejects_inactive_classic_color_state() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(duo_config());

    state
        .board
        .place_mask(PlayerColor::Blue, BoardMask::from_index(index(0, 0)));
    assert_eq!(
        validate_game_state(&state),
        Err(DomainError::from(EngineError::CorruptedState))
    );
}

#[test]
fn duo_incremental_hash_matches_full_hash_after_moves() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(duo_config());
    let state = apply_monomino(
        engine,
        &state,
        PlayerColor::Black,
        player_id(1),
        index(4, 4),
    );
    assert_eq!(state.hash, compute_hash_full(&state));
    let state = apply_monomino(
        engine,
        &state,
        PlayerColor::White,
        player_id(2),
        index(9, 9),
    );
    assert_eq!(state.hash, compute_hash_full(&state));
}

#[test]
fn duo_scoring_uses_advanced_rulebook_points() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(duo_config());
    state.status = blocus_core::GameStatus::Finished;

    let scores = engine
        .score_game(&state, ScoringMode::Advanced)
        .unwrap_or_else(|error| panic!("Duo scoring should succeed: {error}"));
    assert_eq!(scores.entries[0].score, -89);
    assert_eq!(scores.entries[1].score, -89);

    state
        .board
        .place_mask(PlayerColor::Black, first_duo_cells(89));
    state.inventories[PlayerColor::Black.index()] =
        blocus_core::PieceInventory::from_used_mask(ALL_PIECES_MASK);
    state
        .last_piece_by_color
        .set(PlayerColor::Black, piece_id(1));
    state.hash = compute_hash_full(&state);
    let scores = engine
        .score_game(&state, ScoringMode::Advanced)
        .unwrap_or_else(|error| panic!("Duo scoring should succeed: {error}"));
    assert_eq!(scores.entries[0].score, 15);

    state
        .last_piece_by_color
        .set(PlayerColor::Black, piece_id(0));
    state.hash = compute_hash_full(&state);
    let scores = engine
        .score_game(&state, ScoringMode::Advanced)
        .unwrap_or_else(|error| panic!("Duo scoring should succeed: {error}"));
    assert_eq!(scores.entries[0].score, 20);
}
