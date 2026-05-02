use blocus_core::{
    ALL_PIECES_MASK, BlocusEngine, GameConfig, GameId, GameMode, GameState, GameStatus,
    PIECE_COUNT, PieceId, PieceInventory, PlayerColor, PlayerId, PlayerSlots, ScoringMode,
    SharedColorTurn, TurnOrder, compute_hash_full, standard_repository,
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

fn monomino() -> PieceId {
    match PieceId::try_new(0) {
        Ok(piece_id) => piece_id,
        Err(error) => panic!("piece id 0 should be valid: {error}"),
    }
}

fn completed_inventory() -> PieceInventory {
    PieceInventory::from_used_mask(ALL_PIECES_MASK)
}

fn total_piece_square_count() -> i16 {
    standard_repository()
        .pieces()
        .iter()
        .map(|piece| i16::from(piece.square_count()))
        .sum()
}

fn initialized_three_player_state() -> (BlocusEngine, GameState, PlayerId, PlayerId, PlayerId) {
    let engine = BlocusEngine::new();

    let player_one = player_id(1);
    let player_two = player_id(2);
    let player_three = player_id(3);

    let shared = match SharedColorTurn::try_new(
        PlayerColor::Green,
        [player_one, player_two, player_three],
    ) {
        Ok(shared) => shared,
        Err(error) => panic!("three distinct players should form valid shared turn: {error}"),
    };

    let slots = match PlayerSlots::three_player(
        [
            (PlayerColor::Blue, player_one),
            (PlayerColor::Yellow, player_two),
            (PlayerColor::Red, player_three),
        ],
        shared,
    ) {
        Ok(slots) => slots,
        Err(error) => panic!("valid three-player slots should initialize: {error}"),
    };

    let config = match GameConfig::try_new(
        game_id(200),
        GameMode::ThreePlayer,
        ScoringMode::Advanced,
        TurnOrder::OFFICIAL_FIXED,
        slots,
    ) {
        Ok(config) => config,
        Err(error) => panic!("valid three-player config should initialize: {error}"),
    };

    let state = engine.initialize_game(config);

    (engine, state, player_one, player_two, player_three)
}

fn score_for_player(state: &GameState, player: PlayerId) -> i16 {
    let engine = BlocusEngine::new();

    let scoreboard = match engine.score_game(state, ScoringMode::Advanced) {
        Ok(scoreboard) => scoreboard,
        Err(error) => panic!("finished state should be scoreable: {error}"),
    };

    scoreboard
        .entries
        .iter()
        .find(|entry| entry.player_id == player)
        .map_or_else(
            || panic!("missing score entry for player {player}"),
            |entry| entry.score,
        )
}

#[test]
fn three_player_advanced_score_ignores_shared_color_even_if_shared_color_completed_with_monomino_last()
 {
    let (engine, mut state, player_one, player_two, player_three) =
        initialized_three_player_state();

    state.status = GameStatus::Finished;

    state.inventories[PlayerColor::Green.index()] = completed_inventory();

    state
        .last_piece_by_color
        .set(PlayerColor::Green, monomino());

    state.hash = compute_hash_full(&state);

    let scoreboard = match engine.score_game(&state, ScoringMode::Advanced) {
        Ok(scoreboard) => scoreboard,
        Err(error) => panic!("finished state should be scoreable: {error}"),
    };

    assert_eq!(scoreboard.entries.len(), 3);

    let expected_unfinished_color_score = -total_piece_square_count();

    assert_eq!(
        score_for_player(&state, player_one),
        expected_unfinished_color_score
    );
    assert_eq!(
        score_for_player(&state, player_two),
        expected_unfinished_color_score
    );
    assert_eq!(
        score_for_player(&state, player_three),
        expected_unfinished_color_score
    );

    assert!(
        scoreboard.entries.iter().all(|entry| entry.score != 20),
        "shared green completed with monomino-last, but the shared color score must not leak into the three-player scoreboard"
    );
}

#[test]
fn completed_inventory_uses_all_official_piece_bits() {
    assert_eq!(PIECE_COUNT, 21);
    assert_eq!(completed_inventory().used_mask(), ALL_PIECES_MASK);
}
