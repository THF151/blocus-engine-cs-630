use blocus_core::{
    BlocusEngine, BoardIndex, BoardMask, Command, CommandId, DomainError, DomainEventKind,
    GameConfig, GameId, GameMode, GameStatus, PassCommand, PieceId, PlayerColor, PlayerId,
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

fn orientation_id(value: u8) -> blocus_core::OrientationId {
    blocus_core::OrientationId::try_new(value)
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

fn four_player_config() -> GameConfig {
    let slots = PlayerSlots::four_player([
        (PlayerColor::Blue, player_id(1)),
        (PlayerColor::Yellow, player_id(2)),
        (PlayerColor::Red, player_id(3)),
        (PlayerColor::Green, player_id(4)),
    ])
    .unwrap_or_else(|_| panic!("four-player slots should be valid"));

    GameConfig::try_new(
        game_id(100),
        GameMode::FourPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    )
    .unwrap_or_else(|_| panic!("four-player config should be valid"))
}

fn pass_command(color: PlayerColor, player_id: PlayerId) -> PassCommand {
    PassCommand {
        command_id: command_id(1),
        game_id: game_id(100),
        player_id,
        color,
    }
}

fn no_move_blue_state() -> blocus_core::GameState {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(two_player_config());

    state.board.place_mask(
        PlayerColor::Yellow,
        BoardMask::from_index(index(0, 0))
            .union(BoardMask::from_index(index(0, 1)))
            .union(BoardMask::from_index(index(1, 0)))
            .union(BoardMask::from_index(index(1, 1))),
    );
    state.hash = blocus_core::compute_hash_full(&state);

    state
}

#[test]
fn pass_is_rejected_when_current_color_has_a_legal_move() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let command = pass_command(PlayerColor::Blue, player_id(1));

    assert_eq!(
        engine.apply(&state, Command::Pass(command)),
        Err(DomainError::from(
            RuleViolation::PassNotAllowedBecauseMoveExists
        ))
    );

    assert_eq!(state.turn.current_color(), PlayerColor::Blue);
    assert_eq!(state.version, StateVersion::INITIAL);
}

#[test]
fn pass_is_rejected_for_wrong_turn() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let command = pass_command(PlayerColor::Yellow, player_id(2));

    assert_eq!(
        engine.apply(&state, Command::Pass(command)),
        Err(DomainError::from(RuleViolation::WrongPlayerTurn))
    );
}

#[test]
fn pass_is_rejected_for_uncontrolled_color() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let command = pass_command(PlayerColor::Blue, player_id(2));

    assert_eq!(
        engine.apply(&state, Command::Pass(command)),
        Err(DomainError::from(RuleViolation::PlayerDoesNotControlColor))
    );
}

#[test]
fn pass_is_rejected_after_game_finished() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(two_player_config());
    state.status = GameStatus::Finished;

    let command = pass_command(PlayerColor::Blue, player_id(1));

    assert_eq!(
        engine.apply(&state, Command::Pass(command)),
        Err(DomainError::from(RuleViolation::GameAlreadyFinished))
    );
}

#[test]
fn pass_is_allowed_when_current_color_has_no_legal_move() {
    let engine = BlocusEngine::new();
    let state = no_move_blue_state();

    assert_eq!(
        engine.has_any_valid_move(&state, player_id(1), PlayerColor::Blue),
        Ok(false)
    );

    let command = pass_command(PlayerColor::Blue, player_id(1));
    let result = engine
        .apply(&state, Command::Pass(command))
        .unwrap_or_else(|error| panic!("pass should be legal when no move exists: {error}"));

    assert!(result.next_state.turn.is_passed(PlayerColor::Blue));
    assert_eq!(result.next_state.version, StateVersion::new(1));
    assert_eq!(result.events[0].kind, DomainEventKind::PlayerPassed);
    assert_eq!(
        result.response.kind,
        blocus_core::DomainResponseKind::PlayerPassed
    );
    assert_eq!(result.response.message, "player passed");
}

#[test]
fn successful_pass_does_not_mutate_original_state() {
    let engine = BlocusEngine::new();
    let state = no_move_blue_state();
    let original = state.clone();

    let command = pass_command(PlayerColor::Blue, player_id(1));
    let result = engine
        .apply(&state, Command::Pass(command))
        .unwrap_or_else(|error| panic!("pass should be legal when no move exists: {error}"));

    assert_eq!(state, original);
    assert!(!state.turn.is_passed(PlayerColor::Blue));
    assert!(result.next_state.turn.is_passed(PlayerColor::Blue));
}

#[test]
fn pass_finishes_game_when_no_unpassed_color_has_any_legal_move() {
    let engine = BlocusEngine::new();
    let mut state = no_move_blue_state();

    state.turn = state
        .turn
        .marked_passed(PlayerColor::Yellow)
        .marked_passed(PlayerColor::Red)
        .marked_passed(PlayerColor::Green);
    state.hash = blocus_core::compute_hash_full(&state);

    let command = pass_command(PlayerColor::Blue, player_id(1));
    let result = engine
        .apply(&state, Command::Pass(command))
        .unwrap_or_else(|error| panic!("final pass should be legal: {error}"));

    assert_eq!(result.next_state.status, GameStatus::Finished);
    assert!(result.next_state.turn.all_colors_passed());
    assert_eq!(
        result
            .events
            .iter()
            .map(|event| event.kind)
            .collect::<Vec<_>>(),
        vec![DomainEventKind::PlayerPassed, DomainEventKind::GameFinished]
    );
}

#[test]
fn place_can_finish_game_when_no_unpassed_color_can_move_after_transition() {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(four_player_config());

    state.turn = state
        .turn
        .marked_passed(PlayerColor::Yellow)
        .marked_passed(PlayerColor::Red)
        .marked_passed(PlayerColor::Green);

    // Leave only Blue's I1 available. After this placement, Blue has consumed
    // its final piece. Since every other color is already passed, no unpassed
    // color can move after the transition and the game must finish.
    for raw_piece_id in 1..blocus_core::PIECE_COUNT {
        state.inventories[PlayerColor::Blue.index()].mark_used(piece_id(raw_piece_id));
    }
    state.hash = blocus_core::compute_hash_full(&state);

    assert_eq!(
        engine.has_any_valid_move(&state, player_id(1), PlayerColor::Blue),
        Ok(true)
    );

    let command = blocus_core::PlaceCommand {
        command_id: command_id(2),
        game_id: game_id(100),
        player_id: player_id(1),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        anchor: index(0, 0),
    };

    let result = engine
        .apply(&state, Command::Place(command))
        .unwrap_or_else(|error| panic!("final placement should be legal: {error}"));

    assert_eq!(result.next_state.status, GameStatus::Finished);
    assert!(result.next_state.inventories[PlayerColor::Blue.index()].is_complete());
    assert_eq!(
        result
            .events
            .iter()
            .map(|event| event.kind)
            .collect::<Vec<_>>(),
        vec![DomainEventKind::MoveApplied, DomainEventKind::GameFinished]
    );
}
