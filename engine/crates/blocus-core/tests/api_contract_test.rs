use blocus_core::{
    BoardIndex, BoardState, Command, CommandId, DomainEvent, DomainEventKind, DomainResponse,
    DomainResponseKind, GameId, GameResult, GameStatus, LegalMove, OrientationId, PassCommand,
    PieceId, PlaceCommand, PlayerColor, PlayerId, ScoreBoard, ScoreEntry, ScoringMode,
    StateSchemaVersion, StateVersion, ZobristHash,
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
    let Ok(piece) = PieceId::try_new(value) else {
        panic!("piece id {value} should be valid");
    };

    piece
}

fn orientation_id(value: u8) -> OrientationId {
    let Ok(orientation) = OrientationId::try_new(value) else {
        panic!("orientation id {value} should be valid");
    };

    orientation
}

fn board_index(row: u8, col: u8) -> BoardIndex {
    let Ok(index) = BoardIndex::from_row_col(row, col) else {
        panic!("row {row} col {col} should be valid");
    };

    index
}

fn game_state(version: u64) -> blocus_core::GameState {
    blocus_core::GameState {
        schema_version: StateSchemaVersion::CURRENT,
        board: BoardState::EMPTY,
        status: GameStatus::InProgress,
        version: StateVersion::new(version),
        hash: ZobristHash::new(version),
    }
}

#[test]
fn state_schema_version_current_default_new_and_raw_value_are_stable() {
    let current = StateSchemaVersion::CURRENT;
    let explicit = StateSchemaVersion::new(1);
    let default = StateSchemaVersion::default();

    assert_eq!(current, explicit);
    assert_eq!(current, default);
    assert_eq!(current.as_u16(), 1);
    assert_eq!(u16::from(current), 1);
}

#[test]
fn state_schema_version_is_copy_ordered_and_hashable() {
    let first = StateSchemaVersion::new(1);
    let duplicate = StateSchemaVersion::new(1);
    let second = StateSchemaVersion::new(2);
    let copied = first;

    assert_eq!(first, copied);
    assert_eq!(first, duplicate);
    assert_ne!(first, second);
    assert!(first < second);

    let mut versions = HashSet::new();
    versions.insert(first);
    versions.insert(duplicate);
    versions.insert(second);

    assert_eq!(versions.len(), 2);
    assert!(versions.contains(&StateSchemaVersion::new(1)));
    assert!(versions.contains(&StateSchemaVersion::new(2)));
}

#[test]
fn game_status_variants_are_copy_ordered_and_hashable() {
    let active = GameStatus::InProgress;
    let duplicate_active = GameStatus::InProgress;
    let finished = GameStatus::Finished;
    let copied = active;

    assert_eq!(active, copied);
    assert_eq!(active, duplicate_active);
    assert_ne!(active, finished);
    assert!(active < finished);
    assert!(format!("{active:?}").contains("InProgress"));

    let mut statuses = HashSet::new();
    statuses.insert(active);
    statuses.insert(duplicate_active);
    statuses.insert(finished);

    assert_eq!(statuses.len(), 2);
    assert!(statuses.contains(&GameStatus::InProgress));
    assert!(statuses.contains(&GameStatus::Finished));
}

#[test]
fn scoring_mode_variants_are_copy_ordered_and_hashable() {
    let basic = ScoringMode::Basic;
    let duplicate_basic = ScoringMode::Basic;
    let advanced = ScoringMode::Advanced;
    let copied = basic;

    assert_eq!(basic, copied);
    assert_eq!(basic, duplicate_basic);
    assert_ne!(basic, advanced);
    assert!(basic < advanced);
    assert!(format!("{basic:?}").contains("Basic"));

    let mut modes = HashSet::new();
    modes.insert(basic);
    modes.insert(duplicate_basic);
    modes.insert(advanced);

    assert_eq!(modes.len(), 2);
    assert!(modes.contains(&ScoringMode::Basic));
    assert!(modes.contains(&ScoringMode::Advanced));
}

#[test]
fn place_command_is_a_typed_value_object() {
    let command = PlaceCommand {
        command_id: command_id(1),
        game_id: game_id(2),
        player_id: player_id(3),
        color: PlayerColor::Blue,
        piece_id: piece_id(4),
        orientation_id: orientation_id(5),
        anchor: board_index(6, 7),
    };

    let copied = command;

    assert_eq!(command, copied);
    assert_eq!(command.command_id, command_id(1));
    assert_eq!(command.game_id, game_id(2));
    assert_eq!(command.player_id, player_id(3));
    assert_eq!(command.color, PlayerColor::Blue);
    assert_eq!(command.piece_id, piece_id(4));
    assert_eq!(command.orientation_id, orientation_id(5));
    assert_eq!(command.anchor, board_index(6, 7));
    assert!(format!("{command:?}").contains("PlaceCommand"));
}

#[test]
fn pass_command_is_a_typed_value_object() {
    let command = PassCommand {
        command_id: command_id(1),
        game_id: game_id(2),
        player_id: player_id(3),
        color: PlayerColor::Yellow,
    };

    let copied = command;

    assert_eq!(command, copied);
    assert_eq!(command.command_id, command_id(1));
    assert_eq!(command.game_id, game_id(2));
    assert_eq!(command.player_id, player_id(3));
    assert_eq!(command.color, PlayerColor::Yellow);
    assert!(format!("{command:?}").contains("PassCommand"));
}

#[test]
fn command_enum_preserves_place_and_pass_payloads() {
    let place_payload = PlaceCommand {
        command_id: command_id(1),
        game_id: game_id(2),
        player_id: player_id(3),
        color: PlayerColor::Blue,
        piece_id: piece_id(0),
        orientation_id: orientation_id(0),
        anchor: board_index(0, 0),
    };

    let pass_payload = PassCommand {
        command_id: command_id(4),
        game_id: game_id(2),
        player_id: player_id(3),
        color: PlayerColor::Blue,
    };

    let place = Command::Place(place_payload);
    let duplicate_place = Command::Place(place_payload);
    let pass = Command::Pass(pass_payload);
    let copied_place = place;

    assert_eq!(place, copied_place);
    assert_eq!(place, duplicate_place);
    assert_ne!(place, pass);
    assert!(format!("{place:?}").contains("Place"));

    let mut commands = HashSet::new();
    commands.insert(place);
    commands.insert(duplicate_place);
    commands.insert(pass);

    assert_eq!(commands.len(), 2);
    assert!(commands.contains(&Command::Place(place_payload)));
    assert!(commands.contains(&Command::Pass(pass_payload)));
}

#[test]
fn legal_move_is_a_typed_value_object() {
    let legal_move = LegalMove {
        piece_id: piece_id(10),
        orientation_id: orientation_id(2),
        anchor: board_index(3, 4),
        score_delta: 5,
    };

    let copied = legal_move;

    assert_eq!(legal_move, copied);
    assert_eq!(legal_move.piece_id, piece_id(10));
    assert_eq!(legal_move.orientation_id, orientation_id(2));
    assert_eq!(legal_move.anchor, board_index(3, 4));
    assert_eq!(legal_move.score_delta, 5);
    assert!(format!("{legal_move:?}").contains("LegalMove"));

    let mut moves = HashSet::new();
    moves.insert(legal_move);
    moves.insert(copied);

    assert_eq!(moves.len(), 1);
}

#[test]
fn domain_event_kind_variants_are_copy_ordered_and_hashable() {
    let move_applied = DomainEventKind::MoveApplied;
    let duplicate = DomainEventKind::MoveApplied;
    let passed = DomainEventKind::PlayerPassed;
    let turn_advanced = DomainEventKind::TurnAdvanced;
    let game_finished = DomainEventKind::GameFinished;
    let copied = move_applied;

    assert_eq!(move_applied, copied);
    assert_eq!(move_applied, duplicate);
    assert_ne!(move_applied, passed);
    assert!(move_applied < passed);
    assert!(format!("{game_finished:?}").contains("GameFinished"));

    let mut kinds = HashSet::new();
    kinds.insert(move_applied);
    kinds.insert(duplicate);
    kinds.insert(passed);
    kinds.insert(turn_advanced);
    kinds.insert(game_finished);

    assert_eq!(kinds.len(), 4);
    assert!(kinds.contains(&DomainEventKind::MoveApplied));
    assert!(kinds.contains(&DomainEventKind::PlayerPassed));
    assert!(kinds.contains(&DomainEventKind::TurnAdvanced));
    assert!(kinds.contains(&DomainEventKind::GameFinished));
}

#[test]
fn domain_event_is_a_typed_value_object() {
    let event = DomainEvent {
        kind: DomainEventKind::TurnAdvanced,
        game_id: game_id(9),
        version: StateVersion::new(3),
    };

    let duplicate = event.clone();

    assert_eq!(event, duplicate);
    assert_eq!(event.kind, DomainEventKind::TurnAdvanced);
    assert_eq!(event.game_id, game_id(9));
    assert_eq!(event.version, StateVersion::new(3));
    assert!(format!("{event:?}").contains("TurnAdvanced"));

    let mut events = HashSet::new();
    events.insert(event);
    events.insert(duplicate);

    assert_eq!(events.len(), 1);
}

#[test]
fn domain_response_kind_variants_are_copy_ordered_and_hashable() {
    let move_applied = DomainResponseKind::MoveApplied;
    let duplicate = DomainResponseKind::MoveApplied;
    let passed = DomainResponseKind::PlayerPassed;
    let finished = DomainResponseKind::GameFinished;
    let copied = move_applied;

    assert_eq!(move_applied, copied);
    assert_eq!(move_applied, duplicate);
    assert_ne!(move_applied, passed);
    assert!(move_applied < passed);
    assert!(format!("{finished:?}").contains("GameFinished"));

    let mut kinds = HashSet::new();
    kinds.insert(move_applied);
    kinds.insert(duplicate);
    kinds.insert(passed);
    kinds.insert(finished);

    assert_eq!(kinds.len(), 3);
    assert!(kinds.contains(&DomainResponseKind::MoveApplied));
    assert!(kinds.contains(&DomainResponseKind::PlayerPassed));
    assert!(kinds.contains(&DomainResponseKind::GameFinished));
}

#[test]
fn domain_response_is_a_typed_value_object() {
    let response = DomainResponse {
        kind: DomainResponseKind::MoveApplied,
        message: "move applied".to_owned(),
    };

    let duplicate = response.clone();

    assert_eq!(response, duplicate);
    assert_eq!(response.kind, DomainResponseKind::MoveApplied);
    assert_eq!(response.message, "move applied");
    assert!(format!("{response:?}").contains("move applied"));

    let mut responses = HashSet::new();
    responses.insert(response);
    responses.insert(duplicate);

    assert_eq!(responses.len(), 1);
}

#[test]
fn score_entry_is_a_typed_value_object() {
    let entry = ScoreEntry {
        player_id: player_id(42),
        score: -12,
    };

    let copied = entry;

    assert_eq!(entry, copied);
    assert_eq!(entry.player_id, player_id(42));
    assert_eq!(entry.score, -12);
    assert!(format!("{entry:?}").contains("ScoreEntry"));

    let mut entries = HashSet::new();
    entries.insert(entry);
    entries.insert(copied);

    assert_eq!(entries.len(), 1);
}

#[test]
fn scoreboard_is_a_typed_value_object() {
    let entry = ScoreEntry {
        player_id: player_id(42),
        score: 20,
    };

    let scoreboard = ScoreBoard {
        scoring: ScoringMode::Advanced,
        entries: vec![entry],
    };

    let duplicate = scoreboard.clone();

    assert_eq!(scoreboard, duplicate);
    assert_eq!(scoreboard.scoring, ScoringMode::Advanced);
    assert_eq!(scoreboard.entries, vec![entry]);
    assert!(format!("{scoreboard:?}").contains("Advanced"));

    let mut boards = HashSet::new();
    boards.insert(scoreboard);
    boards.insert(duplicate);

    assert_eq!(boards.len(), 1);
}

#[test]
fn game_state_is_a_typed_value_object() {
    let state = game_state(0);
    let duplicate = state.clone();

    assert_eq!(state, duplicate);
    assert_eq!(state.schema_version, StateSchemaVersion::CURRENT);
    assert_eq!(state.board, BoardState::EMPTY);
    assert_eq!(state.status, GameStatus::InProgress);
    assert_eq!(state.version, StateVersion::new(0));
    assert_eq!(state.hash, ZobristHash::ZERO);
    assert!(format!("{state:?}").contains("InProgress"));

    let mut states = HashSet::new();
    states.insert(state);
    states.insert(duplicate);

    assert_eq!(states.len(), 1);
}

#[test]
fn game_result_is_a_typed_value_object() {
    let next_state = game_state(1);
    let event = DomainEvent {
        kind: DomainEventKind::MoveApplied,
        game_id: game_id(100),
        version: StateVersion::new(1),
    };
    let response = DomainResponse {
        kind: DomainResponseKind::MoveApplied,
        message: "move applied".to_owned(),
    };

    let result = GameResult {
        next_state: next_state.clone(),
        events: vec![event.clone()],
        response: response.clone(),
    };

    let duplicate = result.clone();

    assert_eq!(result, duplicate);
    assert_eq!(result.next_state, next_state);
    assert_eq!(result.events, vec![event]);
    assert_eq!(result.response, response);
    assert!(format!("{result:?}").contains("GameResult"));

    let mut results = HashSet::new();
    results.insert(result);
    results.insert(duplicate);

    assert_eq!(results.len(), 1);
}

#[test]
fn game_result_type_is_marked_must_use_by_contract() {
    fn returns_game_result() -> GameResult {
        GameResult {
            next_state: game_state(1),
            events: Vec::new(),
            response: DomainResponse {
                kind: DomainResponseKind::MoveApplied,
                message: String::new(),
            },
        }
    }

    let result = returns_game_result();

    assert_eq!(result.next_state.version, StateVersion::new(1));
    assert!(result.events.is_empty());
    assert_eq!(result.response.kind, DomainResponseKind::MoveApplied);
}

const CONST_SCHEMA_CURRENT: StateSchemaVersion = StateSchemaVersion::CURRENT;
const CONST_SCHEMA_EXPLICIT: StateSchemaVersion = StateSchemaVersion::new(1);
const CONST_SCHEMA_RAW: u16 = CONST_SCHEMA_CURRENT.as_u16();

#[test]
fn const_context_apis_work_for_state_schema_version() {
    assert_eq!(CONST_SCHEMA_CURRENT, StateSchemaVersion::CURRENT);
    assert_eq!(CONST_SCHEMA_EXPLICIT, StateSchemaVersion::CURRENT);
    assert_eq!(CONST_SCHEMA_RAW, 1);
}
