use uuid::Uuid;

use blocus_core::{
    BlocusEngine, BoardIndex, Command, CommandId, GameConfig, GameId, GameMode, GameStatus,
    OrientationId, PassCommand, PieceId, PlaceCommand, PlayerColor, PlayerId, PlayerSlots,
    ScoringMode, TurnOrder,
};

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn player(value: u128) -> PlayerId {
    PlayerId::from_uuid(uuid(value))
}

fn game_id(value: u128) -> GameId {
    GameId::from_uuid(uuid(value))
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

fn two_player_slots() -> PlayerSlots {
    PlayerSlots::two_player(player(10), player(20))
        .unwrap_or_else(|_| panic!("two-player slots should be valid"))
}

fn two_player_config() -> GameConfig {
    GameConfig::try_new(
        game_id(1),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        two_player_slots(),
    )
    .unwrap_or_else(|_| panic!("two-player config should be valid"))
}

fn opening_place_command(
    state_game_id: GameId,
    command_value: u128,
    player_id: PlayerId,
    color: PlayerColor,
    piece_id: PieceId,
    orientation_id: OrientationId,
    anchor: BoardIndex,
) -> Command {
    Command::Place(PlaceCommand {
        command_id: command_id(command_value),
        game_id: state_game_id,
        player_id,
        color,
        piece_id,
        orientation_id,
        anchor,
    })
}

fn pass_command(
    state_game_id: GameId,
    command_value: u128,
    player_id: PlayerId,
    color: PlayerColor,
) -> Command {
    Command::Pass(PassCommand {
        command_id: command_id(command_value),
        game_id: state_game_id,
        player_id,
        color,
    })
}

#[test]
fn board_state_exposes_color_at_counts_and_occupied_cells() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    assert!(state.board.is_empty());
    assert_eq!(state.board.occupied_count(PlayerColor::Blue), 0);
    assert_eq!(state.board.occupied_cells(PlayerColor::Blue), Vec::new());
    assert_eq!(state.board.occupied_cells_all(), Vec::new());

    let result = engine
        .apply(
            &state,
            opening_place_command(
                state.game_id,
                1,
                player(10),
                PlayerColor::Blue,
                piece_id(0),
                orientation_id(0),
                index(0, 0),
            ),
        )
        .unwrap_or_else(|error| panic!("first blue move should be legal: {error}"));

    let next = result.next_state;
    let corner = index(0, 0);

    assert!(!next.board.is_empty());
    assert_eq!(next.board.color_at(corner), Some(PlayerColor::Blue));
    assert_eq!(next.board.occupied_count(PlayerColor::Blue), 1);
    assert_eq!(next.board.occupied_count(PlayerColor::Yellow), 0);

    assert_eq!(next.board.occupied_cells(PlayerColor::Blue), vec![corner]);
    assert_eq!(
        next.board.occupied_cells_all(),
        vec![(PlayerColor::Blue, corner)]
    );
}

#[test]
fn engine_exposes_piece_repository_metadata() {
    let engine = BlocusEngine::new();

    let pieces = engine.pieces();
    assert_eq!(pieces.len(), usize::from(blocus_core::PIECE_COUNT));

    let monomino = engine.piece(piece_id(0));

    assert_eq!(monomino.id(), piece_id(0));
    assert_eq!(monomino.name(), "I1");
    assert_eq!(monomino.square_count(), 1);
    assert_eq!(monomino.orientation_count(), 1);

    let orientation = monomino
        .orientation(orientation_id(0))
        .unwrap_or_else(|| panic!("monomino orientation 0 should exist"));

    assert_eq!(orientation.id(), orientation_id(0));
    assert_eq!(orientation.shape().square_count(), 1);
    assert_eq!(orientation.shape().cells(), vec![(0, 0)]);
}

#[test]
fn engine_exposes_all_valid_moves_and_piece_filtered_valid_moves() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let blue_player = player(10);
    let monomino = piece_id(0);

    let all_moves = engine
        .get_valid_moves(&state, blue_player, PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("valid move generation should succeed: {error}"));

    let monomino_moves = engine
        .get_valid_moves_for_piece(&state, blue_player, PlayerColor::Blue, monomino)
        .unwrap_or_else(|error| panic!("piece-filtered move generation should succeed: {error}"));

    assert!(!all_moves.is_empty());
    assert!(!monomino_moves.is_empty());
    assert!(
        monomino_moves
            .iter()
            .all(|legal_move| legal_move.piece_id == monomino)
    );
    assert!(monomino_moves.len() <= all_moves.len());

    assert_eq!(
        engine.has_any_valid_move(&state, blue_player, PlayerColor::Blue),
        Ok(true)
    );

    assert_eq!(
        engine.has_any_valid_move_for_piece(&state, blue_player, PlayerColor::Blue, monomino),
        Ok(true)
    );
}

#[test]
fn valid_moves_are_empty_for_wrong_player_or_finished_game() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let wrong_player_moves = engine
        .get_valid_moves(&state, player(20), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("wrong controller should not corrupt movegen: {error}"));

    assert!(wrong_player_moves.is_empty());

    let mut finished = state.clone();
    finished.status = GameStatus::Finished;

    let finished_moves = engine
        .get_valid_moves(&finished, player(10), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("finished game should not corrupt movegen: {error}"));

    assert!(finished_moves.is_empty());
}

#[test]
fn piece_inventory_tracks_used_and_available_pieces_after_move() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let monomino = piece_id(0);

    assert!(!state.inventories[PlayerColor::Blue.index()].is_used(monomino));
    assert_eq!(
        state.used_piece_ids(PlayerColor::Blue),
        Vec::<PieceId>::new()
    );
    assert_eq!(state.available_piece_ids(PlayerColor::Blue).len(), 21);

    let result = engine
        .apply(
            &state,
            opening_place_command(
                state.game_id,
                2,
                player(10),
                PlayerColor::Blue,
                monomino,
                orientation_id(0),
                index(0, 0),
            ),
        )
        .unwrap_or_else(|error| panic!("first blue move should be legal: {error}"));

    let next = result.next_state;

    assert!(next.inventories[PlayerColor::Blue.index()].is_used(monomino));
    assert_eq!(next.inventories[PlayerColor::Blue.index()].used_count(), 1);
    assert_eq!(
        next.inventories[PlayerColor::Blue.index()].available_count(),
        u32::from(blocus_core::PIECE_COUNT) - 1
    );

    assert_eq!(next.used_piece_ids(PlayerColor::Blue), vec![monomino]);
    assert!(
        !next
            .available_piece_ids(PlayerColor::Blue)
            .contains(&monomino)
    );

    let yellow_monomino_moves = engine
        .get_valid_moves_for_piece(&next, player(20), PlayerColor::Yellow, monomino)
        .unwrap_or_else(|error| panic!("yellow monomino move query should succeed: {error}"));

    assert!(!yellow_monomino_moves.is_empty());

    let used_blue_monomino_moves = engine
        .get_valid_moves_for_piece(&next, player(10), PlayerColor::Blue, monomino)
        .unwrap_or_else(|error| panic!("used piece move query should succeed: {error}"));

    assert!(used_blue_monomino_moves.is_empty());
}

#[test]
fn pass_probe_uses_valid_move_generation() {
    let engine = BlocusEngine::new();
    let state = engine.initialize_game(two_player_config());

    let result = engine.apply(
        &state,
        pass_command(state.game_id, 3, player(10), PlayerColor::Blue),
    );

    assert!(
        result.is_err(),
        "passing must be rejected while a legal move exists"
    );
}
