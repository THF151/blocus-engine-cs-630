use blocus_core::{
    BoardIndex, BoardMask, DomainError, GameId, GameMode, GameState, GameStatus, InputError,
    OrientationId, PassCommand, PieceId, PlaceCommand, PlayerColor, PlayerId, PlayerSlots,
    RuleViolation, ScoringMode, StateVersion, TurnOrder, TurnState, ZobristHash, build_placement,
    standard_piece, standard_repository, validate_place_command,
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

fn piece_id(value: u8) -> PieceId {
    let Ok(piece_id) = PieceId::try_new(value) else {
        panic!("piece id should be valid");
    };

    piece_id
}

fn orientation_id(value: u8) -> OrientationId {
    let Ok(orientation_id) = OrientationId::try_new(value) else {
        panic!("orientation id should be valid");
    };

    orientation_id
}

fn board_index(row: u8, col: u8) -> BoardIndex {
    let Ok(index) = BoardIndex::from_row_col(row, col) else {
        panic!("board index should be valid");
    };

    index
}

fn two_player_slots() -> PlayerSlots {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    slots
}

fn state() -> GameState {
    GameState {
        schema_version: blocus_core::StateSchemaVersion::CURRENT,
        game_id: game_id(10),
        mode: GameMode::TwoPlayer,
        scoring: ScoringMode::Basic,
        turn_order: TurnOrder::OFFICIAL_FIXED,
        player_slots: two_player_slots(),
        board: blocus_core::BoardState::EMPTY,
        inventories: [blocus_core::PieceInventory::EMPTY; blocus_core::PLAYER_COLOR_COUNT],
        turn: TurnState::new(TurnOrder::OFFICIAL_FIXED),
        status: GameStatus::InProgress,
        version: StateVersion::INITIAL,
        hash: ZobristHash::ZERO,
    }
}

fn valid_place_command() -> PlaceCommand {
    PlaceCommand {
        command_id: blocus_core::CommandId::from_uuid(uuid(99)),
        game_id: game_id(10),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        anchor: board_index(0, 0),
    }
}

#[test]
fn build_placement_builds_mask_at_origin() {
    let piece = standard_piece(piece_id(3));
    let Some(orientation) = piece.orientation(orientation_id(0)) else {
        panic!("orientation should exist");
    };

    let Ok(placement) = build_placement(piece.id(), orientation, board_index(0, 0)) else {
        panic!("placement should fit at origin");
    };

    assert_eq!(placement.piece_id(), piece.id());
    assert_eq!(placement.orientation_id(), orientation_id(0));
    assert_eq!(placement.anchor(), board_index(0, 0));
    assert_eq!(placement.square_count(), 3);

    assert!(placement.mask().contains(board_index(0, 0)));
    assert!(placement.mask().contains(board_index(1, 0)));
    assert!(placement.mask().contains(board_index(1, 1)));
    assert_eq!(placement.mask().count(), 3);
}

#[test]
fn build_placement_handles_lane_boundary_crossing() {
    let piece = standard_piece(piece_id(1));
    let Some(orientation) = piece.orientation(orientation_id(1)) else {
        panic!("vertical I2 orientation should exist");
    };

    let Ok(placement) = build_placement(piece.id(), orientation, board_index(3, 19)) else {
        panic!("placement should fit across u128 lane boundary");
    };

    assert!(placement.mask().contains(board_index(3, 19)));
    assert!(placement.mask().contains(board_index(4, 19)));
    assert_eq!(placement.mask().count(), 2);
}

#[test]
fn build_placement_rejects_right_edge_out_of_bounds() {
    let piece = standard_piece(piece_id(1));
    let Some(orientation) = piece.orientation(orientation_id(0)) else {
        panic!("horizontal I2 orientation should exist");
    };

    assert_eq!(
        build_placement(piece.id(), orientation, board_index(0, 19)),
        Err(DomainError::from(RuleViolation::OutOfBounds))
    );
}

#[test]
fn build_placement_rejects_bottom_edge_out_of_bounds() {
    let piece = standard_piece(piece_id(1));
    let Some(orientation) = piece.orientation(orientation_id(1)) else {
        panic!("vertical I2 orientation should exist");
    };

    assert_eq!(
        build_placement(piece.id(), orientation, board_index(19, 0)),
        Err(DomainError::from(RuleViolation::OutOfBounds))
    );
}

#[test]
fn validate_place_command_accepts_scaffold_valid_empty_board_placement() {
    let state = state();
    let command = valid_place_command();

    let Ok(placement) = validate_place_command(&state, command, standard_repository()) else {
        panic!("valid scaffold placement should pass");
    };

    assert_eq!(placement.mask().count(), 1);
    assert!(placement.mask().contains(board_index(0, 0)));
}

#[test]
fn validate_place_command_rejects_game_id_mismatch() {
    let state = state();
    let mut command = valid_place_command();
    command.game_id = game_id(11);

    assert_eq!(
        validate_place_command(&state, command, standard_repository()),
        Err(DomainError::from(InputError::GameIdMismatch))
    );
}

#[test]
fn validate_place_command_rejects_finished_game() {
    let mut state = state();
    state.status = GameStatus::Finished;

    assert_eq!(
        validate_place_command(&state, valid_place_command(), standard_repository()),
        Err(DomainError::from(RuleViolation::GameAlreadyFinished))
    );
}

#[test]
fn validate_place_command_rejects_player_without_color_control() {
    let state = state();
    let mut command = valid_place_command();
    command.player_id = player_id(2);

    assert_eq!(
        validate_place_command(&state, command, standard_repository()),
        Err(DomainError::from(RuleViolation::PlayerDoesNotControlColor))
    );
}

#[test]
fn validate_place_command_rejects_wrong_color_turn() {
    let state = state();
    let mut command = valid_place_command();
    command.color = PlayerColor::Red;

    assert_eq!(
        validate_place_command(&state, command, standard_repository()),
        Err(DomainError::from(RuleViolation::WrongPlayerTurn))
    );
}

#[test]
fn validate_place_command_rejects_used_piece() {
    let mut state = state();
    state.inventories[PlayerColor::Blue.index()].mark_used(piece_id(0));

    assert_eq!(
        validate_place_command(&state, valid_place_command(), standard_repository()),
        Err(DomainError::from(RuleViolation::PieceAlreadyUsed))
    );
}

#[test]
fn validate_place_command_rejects_unknown_orientation_for_piece() {
    let state = state();
    let mut command = valid_place_command();
    command.piece_id = piece_id(0);
    command.orientation_id = orientation_id(1);

    assert_eq!(
        validate_place_command(&state, command, standard_repository()),
        Err(DomainError::from(InputError::UnknownOrientation))
    );
}

#[test]
fn validate_place_command_rejects_overlap_without_mutating_state() {
    let mut state = state();
    state.board.place_mask(
        PlayerColor::Yellow,
        BoardMask::from_index(board_index(0, 0)),
    );
    let original_state = state.clone();

    assert_eq!(
        validate_place_command(&state, valid_place_command(), standard_repository()),
        Err(DomainError::from(RuleViolation::Overlap))
    );

    assert_eq!(state, original_state);
}

#[test]
fn pass_command_import_is_not_accidentally_needed_for_placement_tests() {
    let command = PassCommand {
        command_id: blocus_core::CommandId::from_uuid(uuid(1)),
        game_id: game_id(10),
        player_id: player_id(1),
        color: PlayerColor::Blue,
    };

    assert_eq!(command.game_id, game_id(10));
}

#[test]
fn validate_place_command_accepts_non_first_move_with_diagonal_same_color_contact() {
    let mut state = state();
    state
        .board
        .place_mask(PlayerColor::Blue, BoardMask::from_index(board_index(0, 0)));
    state.inventories[PlayerColor::Blue.index()].mark_used(piece_id(0));

    let command = PlaceCommand {
        command_id: blocus_core::CommandId::from_uuid(uuid(100)),
        game_id: game_id(10),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(1),
        orientation_id: orientation_id(0),
        anchor: board_index(1, 1),
    };

    let placement = validate_place_command(&state, command, standard_repository())
        .unwrap_or_else(|error| panic!("diagonal same-color contact should be legal: {error}"));

    assert!(placement.mask().contains(board_index(1, 1)));
    assert!(placement.mask().contains(board_index(1, 2)));
}

#[test]
fn validate_place_command_rejects_non_first_move_without_same_color_corner_contact() {
    let mut state = state();
    state
        .board
        .place_mask(PlayerColor::Blue, BoardMask::from_index(board_index(0, 0)));
    state.inventories[PlayerColor::Blue.index()].mark_used(piece_id(0));

    let command = PlaceCommand {
        command_id: blocus_core::CommandId::from_uuid(uuid(101)),
        game_id: game_id(10),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(1),
        orientation_id: orientation_id(0),
        anchor: board_index(3, 3),
    };

    assert_eq!(
        validate_place_command(&state, command, standard_repository()),
        Err(DomainError::from(RuleViolation::MissingCornerContact))
    );
}

#[test]
fn validate_place_command_ignores_different_color_diagonal_contact() {
    let mut state = state();
    state.board.place_mask(
        PlayerColor::Blue,
        BoardMask::from_index(board_index(10, 10)),
    );
    state.inventories[PlayerColor::Blue.index()].mark_used(piece_id(0));
    state.board.place_mask(
        PlayerColor::Yellow,
        BoardMask::from_index(board_index(0, 0)),
    );

    let command = PlaceCommand {
        command_id: blocus_core::CommandId::from_uuid(uuid(102)),
        game_id: game_id(10),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(1),
        orientation_id: orientation_id(0),
        anchor: board_index(1, 1),
    };

    assert_eq!(
        validate_place_command(&state, command, standard_repository()),
        Err(DomainError::from(RuleViolation::MissingCornerContact))
    );
}

#[test]
fn validate_place_command_rejects_same_color_edge_contact() {
    let mut state = state();
    state
        .board
        .place_mask(PlayerColor::Blue, BoardMask::from_index(board_index(0, 0)));
    state.inventories[PlayerColor::Blue.index()].mark_used(piece_id(0));

    let command = PlaceCommand {
        command_id: blocus_core::CommandId::from_uuid(uuid(103)),
        game_id: game_id(10),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(1),
        orientation_id: orientation_id(0),
        anchor: board_index(0, 1),
    };

    assert_eq!(
        validate_place_command(&state, command, standard_repository()),
        Err(DomainError::from(RuleViolation::IllegalEdgeContact))
    );
}
