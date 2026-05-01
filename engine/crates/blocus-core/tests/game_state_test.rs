use blocus_core::{
    BoardState, GameId, GameMode, GameState, GameStatus, PLAYER_COLOR_COUNT, PieceInventory,
    PlayerId, PlayerSlots, ScoringMode, StateSchemaVersion, StateVersion, TurnOrder, TurnState,
    ZobristHash,
};
use std::mem::size_of;
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

fn player_slots() -> PlayerSlots {
    let Ok(slots) = PlayerSlots::two_player(player_id(1), player_id(2)) else {
        panic!("two-player slots should be valid");
    };

    slots
}

fn state() -> GameState {
    GameState {
        schema_version: StateSchemaVersion::CURRENT,
        game_id: game_id(10),
        mode: GameMode::TwoPlayer,
        scoring: ScoringMode::Advanced,
        turn_order: TurnOrder::OFFICIAL_FIXED,
        player_slots: player_slots(),
        board: BoardState::EMPTY,
        inventories: [PieceInventory::EMPTY; PLAYER_COLOR_COUNT],
        turn: TurnState::new(TurnOrder::OFFICIAL_FIXED),
        status: GameStatus::InProgress,
        version: StateVersion::INITIAL,
        hash: ZobristHash::ZERO,
    }
}

#[test]
fn game_state_contains_real_contract_fields() {
    let state = state();

    assert_eq!(state.schema_version, StateSchemaVersion::CURRENT);
    assert_eq!(state.game_id, game_id(10));
    assert_eq!(state.mode, GameMode::TwoPlayer);
    assert_eq!(state.scoring, ScoringMode::Advanced);
    assert_eq!(state.turn_order, TurnOrder::OFFICIAL_FIXED);
    assert_eq!(state.player_slots, player_slots());
    assert_eq!(state.board, BoardState::EMPTY);
    assert_eq!(
        state.inventories,
        [PieceInventory::EMPTY; PLAYER_COLOR_COUNT]
    );
    assert_eq!(state.turn, TurnState::new(TurnOrder::OFFICIAL_FIXED));
    assert_eq!(state.status, GameStatus::InProgress);
    assert_eq!(state.version, StateVersion::INITIAL);
    assert_eq!(state.hash, ZobristHash::ZERO);
}

#[test]
fn game_state_is_clone_eq_hash_and_debug() {
    let state = state();
    let duplicate = state.clone();

    let mut other = state.clone();
    other.version = StateVersion::new(1);

    assert_eq!(state, duplicate);
    assert_ne!(state, other);
    assert!(format!("{state:?}").contains("GameState"));
}

#[test]
fn game_state_size_is_documented() {
    let size = size_of::<GameState>();

    assert!(size > 0);
    assert!(
        size <= 512,
        "GameState should stay compact enough for AI search cloning; actual size: {size}"
    );
}
