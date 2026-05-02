use blocus_core::{
    BlocusEngine, BoardIndex, Command, CommandId, DomainError, DomainEventKind, GameConfig, GameId,
    GameMode, GameStatus, InputError, OrientationId, PieceId, PlaceCommand, PlayerColor, PlayerId,
    PlayerSlots, RuleViolation, ScoringMode, StateVersion, TurnOrder,
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

fn four_player_slots() -> PlayerSlots {
    PlayerSlots::four_player([
        (PlayerColor::Blue, player_id(1)),
        (PlayerColor::Yellow, player_id(2)),
        (PlayerColor::Red, player_id(3)),
        (PlayerColor::Green, player_id(4)),
    ])
    .unwrap_or_else(|_| panic!("four-player slots should be valid"))
}

fn four_player_config() -> GameConfig {
    GameConfig::try_new(
        game_id(100),
        GameMode::FourPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        four_player_slots(),
    )
    .unwrap_or_else(|_| panic!("four-player config should be valid"))
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

#[test]
fn apply_place_updates_board_inventory_turn_version_and_events() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

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

    assert!(state.board.is_empty());
    assert_eq!(state.version, StateVersion::INITIAL);
    assert!(!state.inventories[PlayerColor::Blue.index()].is_used(piece_id(0)));

    assert!(!result.next_state.board.is_empty());
    assert!(
        result
            .next_state
            .board
            .occupied(PlayerColor::Blue)
            .contains(index(0, 0))
    );
    assert!(result.next_state.inventories[PlayerColor::Blue.index()].is_used(piece_id(0)));

    assert_eq!(result.next_state.turn.current_color(), PlayerColor::Yellow);
    assert_eq!(result.next_state.version, StateVersion::new(1));
    assert_eq!(result.next_state.status, GameStatus::InProgress);

    assert_eq!(result.events.len(), 2);
    assert_eq!(result.events[0].kind, DomainEventKind::MoveApplied);
    assert_eq!(result.events[0].game_id, state.game_id);
    assert_eq!(result.events[0].version, StateVersion::new(1));
    assert_eq!(result.events[1].kind, DomainEventKind::TurnAdvanced);
    assert_eq!(result.events[1].game_id, state.game_id);
    assert_eq!(result.events[1].version, StateVersion::new(1));

    assert_eq!(
        result.response.kind,
        blocus_core::DomainResponseKind::MoveApplied
    );
    assert_eq!(result.response.message, "move applied");
}

#[test]
fn apply_place_does_not_mutate_original_state() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

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

    assert!(state.board.is_empty());
    assert_eq!(state.turn.current_color(), PlayerColor::Blue);
    assert_eq!(state.version, StateVersion::INITIAL);
    assert!(!state.inventories[PlayerColor::Blue.index()].is_used(piece_id(0)));

    assert!(!result.next_state.board.is_empty());
    assert_eq!(result.next_state.turn.current_color(), PlayerColor::Yellow);
    assert_eq!(result.next_state.version, StateVersion::new(1));
    assert!(result.next_state.inventories[PlayerColor::Blue.index()].is_used(piece_id(0)));
}

#[test]
fn apply_place_returns_validation_error_for_wrong_starting_corner() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

    let command = opening_place_command(
        PlayerColor::Blue,
        player_id(1),
        piece_id(0),
        orientation_id(0),
        index(0, 1),
    );

    assert_eq!(
        engine.apply(&state, Command::Place(command)),
        Err(DomainError::from(RuleViolation::MissingCornerContact))
    );

    assert!(state.board.is_empty());
    assert!(!state.inventories[PlayerColor::Blue.index()].is_used(piece_id(0)));
    assert_eq!(state.version, StateVersion::INITIAL);
}

#[test]
fn apply_place_returns_validation_error_for_wrong_turn() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

    let command = opening_place_command(
        PlayerColor::Yellow,
        player_id(2),
        piece_id(0),
        orientation_id(0),
        index(0, 19),
    );

    assert_eq!(
        engine.apply(&state, Command::Place(command)),
        Err(DomainError::from(RuleViolation::WrongPlayerTurn))
    );

    assert!(state.board.is_empty());
}

#[test]
fn apply_place_returns_validation_error_for_uncontrolled_color() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

    let command = opening_place_command(
        PlayerColor::Blue,
        player_id(2),
        piece_id(0),
        orientation_id(0),
        index(0, 0),
    );

    assert_eq!(
        engine.apply(&state, Command::Place(command)),
        Err(DomainError::from(RuleViolation::PlayerDoesNotControlColor))
    );

    assert!(state.board.is_empty());
}

#[test]
fn apply_place_returns_validation_error_for_game_id_mismatch() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

    let command = PlaceCommand {
        command_id: command_id(1),
        game_id: game_id(999),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        anchor: index(0, 0),
    };

    assert_eq!(
        engine.apply(&state, Command::Place(command)),
        Err(DomainError::from(InputError::GameIdMismatch))
    );

    assert!(state.board.is_empty());
}
