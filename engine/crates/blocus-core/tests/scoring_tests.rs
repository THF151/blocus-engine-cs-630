use blocus_core::{
    BlocusEngine, BoardIndex, Command, CommandId, DomainError, GameConfig, GameId, GameMode,
    GameState, GameStatus, OrientationId, PIECE_COUNT, PieceId, PlaceCommand, PlayerColor,
    PlayerId, PlayerSlots, RuleViolation, ScoringMode, TurnOrder,
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

fn three_player_config() -> GameConfig {
    let shared = blocus_core::SharedColorTurn::try_new(
        PlayerColor::Green,
        [player_id(1), player_id(2), player_id(3)],
    )
    .unwrap_or_else(|_| panic!("shared color turn should be valid"));

    let slots = PlayerSlots::three_player(
        [
            (PlayerColor::Blue, player_id(1)),
            (PlayerColor::Yellow, player_id(2)),
            (PlayerColor::Red, player_id(3)),
        ],
        shared,
    )
    .unwrap_or_else(|_| panic!("three-player slots should be valid"));

    GameConfig::try_new(
        game_id(100),
        GameMode::ThreePlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    )
    .unwrap_or_else(|_| panic!("three-player config should be valid"))
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

fn finished_state(config: GameConfig) -> GameState {
    let engine = BlocusEngine::new();
    let mut state = engine.initialize_game(config);
    state.status = GameStatus::Finished;
    state
}

fn mark_used(state: &mut GameState, color: PlayerColor, pieces: &[u8]) {
    for raw_piece in pieces {
        let piece = piece_id(*raw_piece);
        state.inventories[color.index()].mark_used(piece);
        state.last_piece_by_color.set(color, piece);
    }
}

fn mark_all_used(state: &mut GameState, color: PlayerColor, last_piece: u8) {
    for raw_piece_id in 0..PIECE_COUNT {
        let piece = piece_id(raw_piece_id);
        state.inventories[color.index()].mark_used(piece);
        state.last_piece_by_color.set(color, piece);
    }

    state.last_piece_by_color.set(color, piece_id(last_piece));
}

fn score_for(state: &GameState, player_id: PlayerId, scoring: ScoringMode) -> i16 {
    let engine = BlocusEngine::new();
    let scoreboard = engine
        .score_game(state, scoring)
        .unwrap_or_else(|error| panic!("scoring should succeed: {error}"));

    scoreboard
        .entries
        .iter()
        .find(|entry| entry.player_id == player_id)
        .unwrap_or_else(|| panic!("score entry for player {player_id} should exist"))
        .score
}

#[test]
fn score_game_rejects_unfinished_game() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert_eq!(
        engine.score_game(&state, ScoringMode::Basic),
        Err(DomainError::from(RuleViolation::GameNotFinished))
    );
}

#[test]
fn basic_scoring_counts_remaining_squares_for_four_players() {
    let mut state = finished_state(four_player_config());

    mark_used(&mut state, PlayerColor::Blue, &[0]);
    mark_used(&mut state, PlayerColor::Yellow, &[0, 1]);
    mark_used(&mut state, PlayerColor::Red, &[0, 1, 2]);
    mark_used(&mut state, PlayerColor::Green, &[0, 1, 2, 3]);

    assert_eq!(score_for(&state, player_id(1), ScoringMode::Basic), 88);
    assert_eq!(score_for(&state, player_id(2), ScoringMode::Basic), 86);
    assert_eq!(score_for(&state, player_id(3), ScoringMode::Basic), 83);
    assert_eq!(score_for(&state, player_id(4), ScoringMode::Basic), 80);
}

#[test]
fn basic_scoring_aggregates_controlled_colors_for_two_players() {
    let mut state = finished_state(two_player_config());

    mark_used(&mut state, PlayerColor::Blue, &[0]);
    mark_used(&mut state, PlayerColor::Red, &[0, 1]);
    mark_used(&mut state, PlayerColor::Yellow, &[0, 1, 2]);
    mark_used(&mut state, PlayerColor::Green, &[0, 1, 2, 3]);

    assert_eq!(score_for(&state, player_id(1), ScoringMode::Basic), 174);
    assert_eq!(score_for(&state, player_id(2), ScoringMode::Basic), 163);
}

#[test]
fn basic_scoring_ignores_shared_color_for_three_players() {
    let mut state = finished_state(three_player_config());

    mark_used(&mut state, PlayerColor::Blue, &[0]);
    mark_used(&mut state, PlayerColor::Yellow, &[0, 1]);
    mark_used(&mut state, PlayerColor::Red, &[0, 1, 2]);
    mark_all_used(&mut state, PlayerColor::Green, 0);

    assert_eq!(score_for(&state, player_id(1), ScoringMode::Basic), 88);
    assert_eq!(score_for(&state, player_id(2), ScoringMode::Basic), 86);
    assert_eq!(score_for(&state, player_id(3), ScoringMode::Basic), 83);
}

#[test]
fn advanced_scoring_is_negative_remaining_squares_without_completion_bonus() {
    let mut state = finished_state(four_player_config());

    mark_used(&mut state, PlayerColor::Blue, &[0]);
    mark_used(&mut state, PlayerColor::Yellow, &[0, 1]);
    mark_used(&mut state, PlayerColor::Red, &[0, 1, 2]);
    mark_used(&mut state, PlayerColor::Green, &[0, 1, 2, 3]);

    assert_eq!(score_for(&state, player_id(1), ScoringMode::Advanced), -88);
    assert_eq!(score_for(&state, player_id(2), ScoringMode::Advanced), -86);
    assert_eq!(score_for(&state, player_id(3), ScoringMode::Advanced), -83);
    assert_eq!(score_for(&state, player_id(4), ScoringMode::Advanced), -80);
}

#[test]
fn advanced_scoring_awards_completion_bonus() {
    let mut state = finished_state(four_player_config());

    mark_all_used(&mut state, PlayerColor::Blue, 1);

    assert_eq!(score_for(&state, player_id(1), ScoringMode::Advanced), 15);
}

#[test]
fn advanced_scoring_awards_monomino_last_bonus() {
    let mut state = finished_state(four_player_config());

    mark_all_used(&mut state, PlayerColor::Blue, 0);

    assert_eq!(score_for(&state, player_id(1), ScoringMode::Advanced), 20);
}

#[test]
fn advanced_scoring_aggregates_two_player_controlled_colors() {
    let mut state = finished_state(two_player_config());

    mark_all_used(&mut state, PlayerColor::Blue, 0);
    mark_used(&mut state, PlayerColor::Red, &[0]);
    mark_used(&mut state, PlayerColor::Yellow, &[0, 1]);
    mark_used(&mut state, PlayerColor::Green, &[0, 1, 2]);

    assert_eq!(score_for(&state, player_id(1), ScoringMode::Advanced), -68);
    assert_eq!(score_for(&state, player_id(2), ScoringMode::Advanced), -169);
}

#[test]
fn place_apply_records_last_piece_for_advanced_scoring() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(four_player_config());

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

    assert_eq!(
        result.next_state.last_piece_by_color.get(PlayerColor::Blue),
        Some(piece_id(0))
    );
}

#[test]
fn score_board_entries_are_deterministic_player_order() {
    let state = finished_state(four_player_config());
    let engine = BlocusEngine::new();

    let scoreboard = engine
        .score_game(&state, ScoringMode::Basic)
        .unwrap_or_else(|error| panic!("scoring should succeed: {error}"));

    assert_eq!(scoreboard.scoring, ScoringMode::Basic);
    assert_eq!(
        scoreboard
            .entries
            .iter()
            .map(|entry| entry.player_id)
            .collect::<Vec<_>>(),
        vec![player_id(1), player_id(2), player_id(3), player_id(4)]
    );
}
