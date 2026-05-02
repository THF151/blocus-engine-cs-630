use blocus_core::{
    BlocusEngine, Command, CommandId, GameConfig, GameId, GameMode, PieceId, PlaceCommand,
    PlayerColor, PlayerId, PlayerSlots, ScoringMode, TurnOrder,
};
use proptest::prelude::*;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn player(value: u128) -> PlayerId {
    PlayerId::from_uuid(uuid(value))
}

fn command_id(value: u128) -> CommandId {
    CommandId::from_uuid(uuid(value))
}

fn game_id(value: u128) -> GameId {
    GameId::from_uuid(uuid(value))
}

fn piece_id(value: u8) -> PieceId {
    let Ok(piece_id) = PieceId::try_new(value) else {
        panic!("piece id {value} should be valid");
    };

    piece_id
}

fn orientation_id(value: u8) -> blocus_core::OrientationId {
    let Ok(orientation_id) = blocus_core::OrientationId::try_new(value) else {
        panic!("orientation id {value} should be valid");
    };

    orientation_id
}

fn two_player_slots() -> PlayerSlots {
    let Ok(slots) = PlayerSlots::two_player(player(1), player(2)) else {
        panic!("two-player slots should be valid");
    };

    slots
}

fn two_player_config_with_game_id(game_id_value: u128) -> GameConfig {
    let Ok(config) = GameConfig::try_new(
        game_id(game_id_value),
        GameMode::TwoPlayer,
        ScoringMode::Basic,
        TurnOrder::OFFICIAL_FIXED,
        two_player_slots(),
    ) else {
        panic!("two-player config should be valid");
    };

    config
}

fn command_from_legal_move(
    command_value: u128,
    state: &blocus_core::GameState,
    player_id: PlayerId,
    color: PlayerColor,
    legal_move: blocus_core::LegalMove,
) -> Command {
    Command::Place(PlaceCommand {
        command_id: command_id(command_value),
        game_id: state.game_id,
        player_id,
        color,
        piece_id: legal_move.piece_id,
        orientation_id: legal_move.orientation_id,
        anchor: legal_move.anchor,
    })
}

fn first_blue_legal_move(
    engine: BlocusEngine,
    state: &blocus_core::GameState,
) -> blocus_core::LegalMove {
    engine
        .valid_moves_iter(state, player(1), PlayerColor::Blue)
        .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
        .next()
        .unwrap_or_else(|| panic!("initial blue state should have a legal move"))
}

proptest! {
    #[test]
    fn repeated_initialization_is_semantically_stable(game_id_value in 1u128..10_000u128) {
        let engine = BlocusEngine::new();

        let first = engine.initialize_game(two_player_config_with_game_id(game_id_value));
        let second = engine.initialize_game(two_player_config_with_game_id(game_id_value));

        prop_assert_eq!(&first, &second);
        prop_assert_eq!(first.hash, second.hash);
        prop_assert_eq!(first.hash, blocus_core::compute_hash_full(&first));
        prop_assert_eq!(second.hash, blocus_core::compute_hash_full(&second));
    }

    #[test]
    fn different_game_ids_do_not_change_position_hashes(
        first_game_id in 1u128..10_000u128,
        second_game_id in 10_001u128..20_000u128,
    ) {
        prop_assume!(first_game_id != second_game_id);

        let engine = BlocusEngine::new();

        let first = engine.initialize_game(two_player_config_with_game_id(first_game_id));
        let second = engine.initialize_game(two_player_config_with_game_id(second_game_id));

        prop_assert_ne!(first.game_id, second.game_id);
        prop_assert_eq!(first.hash, second.hash);
    }

    #[test]
    fn initialized_state_hash_matches_full_recomputation(game_id_value in 1u128..10_000u128) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(game_id_value));

        prop_assert_eq!(state.hash, blocus_core::compute_hash_full(&state));
    }

    #[test]
    fn legal_move_candidates_apply_successfully_without_mutating_original_state(
        take_index in 0usize..58usize,
    ) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(100));
        let original = state.clone();

        let legal_move = engine
            .valid_moves_iter(&state, player(1), PlayerColor::Blue)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .nth(take_index)
            .unwrap_or_else(|| panic!("opening move index should be available"));

        let command = command_from_legal_move(
            1,
            &state,
            player(1),
            PlayerColor::Blue,
            legal_move,
        );

        let result = engine
            .apply(&state, command)
            .unwrap_or_else(|error| panic!("legal move should apply: {error}"));

        let next_state = result.next_state;

        prop_assert_eq!(&state, &original);
        prop_assert_ne!(&next_state, &state);
        prop_assert_eq!(next_state.version.as_u64(), state.version.as_u64() + 1);
        prop_assert_eq!(next_state.hash, blocus_core::compute_hash_full(&next_state));
    }

    #[test]
    fn successful_place_updates_hash_to_full_recomputation(take_index in 0usize..58usize) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(100));

        let legal_move = engine
            .valid_moves_iter(&state, player(1), PlayerColor::Blue)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .nth(take_index)
            .unwrap_or_else(|| panic!("opening move index should be available"));

        let command = command_from_legal_move(
            1,
            &state,
            player(1),
            PlayerColor::Blue,
            legal_move,
        );

        let result = engine
            .apply(&state, command)
            .unwrap_or_else(|error| panic!("legal move should apply: {error}"));

        prop_assert_eq!(
            result.next_state.hash,
            blocus_core::compute_hash_full(&result.next_state)
        );
    }

    #[test]
    fn invalid_place_does_not_mutate_original_state(row in 0u8..20u8, col in 0u8..20u8) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(100));
        let original = state.clone();

        let Ok(anchor) = blocus_core::BoardIndex::from_row_col(row, col) else {
            return Ok(());
        };

        let command = Command::Place(PlaceCommand {
            command_id: command_id(1),
            game_id: state.game_id,
            player_id: player(2),
            color: PlayerColor::Yellow,
            piece_id: piece_id(0),
            orientation_id: orientation_id(0),
            anchor,
        });

        let result = engine.apply(&state, command);

        prop_assert!(result.is_err());
        prop_assert_eq!(&state, &original);
        prop_assert_eq!(state.hash, blocus_core::compute_hash_full(&state));
    }

    #[test]
    fn legal_move_generation_excludes_used_piece_after_successful_move(take_index in 0usize..58usize) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(100));

        let legal_move = engine
            .valid_moves_iter(&state, player(1), PlayerColor::Blue)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .nth(take_index)
            .unwrap_or_else(|| panic!("opening move index should be available"));

        let used_piece = legal_move.piece_id;

        let command = command_from_legal_move(
            1,
            &state,
            player(1),
            PlayerColor::Blue,
            legal_move,
        );

        let result = engine
            .apply(&state, command)
            .unwrap_or_else(|error| panic!("legal move should apply: {error}"));

        let next_state = result.next_state;

        let remaining_blue_moves = engine
            .valid_moves_iter(&next_state, player(1), PlayerColor::Blue)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .collect::<Vec<_>>();

        prop_assert!(
            remaining_blue_moves
                .iter()
                .all(|legal_move| legal_move.piece_id != used_piece)
        );
    }

    #[test]
    fn opening_move_generation_is_empty_for_non_current_color(game_id_value in 1u128..10_000u128) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(game_id_value));

        let yellow_moves = engine
            .valid_moves_iter(&state, player(2), PlayerColor::Yellow)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .collect::<Vec<_>>();

        let red_moves = engine
            .valid_moves_iter(&state, player(1), PlayerColor::Red)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .collect::<Vec<_>>();

        let green_moves = engine
            .valid_moves_iter(&state, player(2), PlayerColor::Green)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .collect::<Vec<_>>();

        prop_assert!(yellow_moves.is_empty());
        prop_assert!(red_moves.is_empty());
        prop_assert!(green_moves.is_empty());
    }

    #[test]
    fn opening_move_generation_contains_only_corner_covering_moves_for_current_color(
        game_id_value in 1u128..10_000u128,
    ) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(game_id_value));

        let moves = engine
            .valid_moves_iter(&state, player(1), PlayerColor::Blue)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .collect::<Vec<_>>();

        prop_assert!(!moves.is_empty());

        let corner = blocus_core::BoardIndex::from_row_col(0, 0)
            .unwrap_or_else(|error| panic!("blue corner should be valid: {error}"));

        for legal_move in moves {
            let command = command_from_legal_move(
                1,
                &state,
                player(1),
                PlayerColor::Blue,
                legal_move,
            );

            let result = engine
                .apply(&state, command)
                .unwrap_or_else(|error| panic!("opening legal move should apply: {error}"));

            prop_assert!(
                result
                    .next_state
                    .board
                    .occupied(PlayerColor::Blue)
                    .contains(corner)
            );
        }
    }

    #[test]
    fn place_result_hash_changes_for_successful_move(take_index in 0usize..58usize) {
        let engine = BlocusEngine::new();
        let state = engine.initialize_game(two_player_config_with_game_id(100));
        let original_hash = state.hash;

        let legal_move = engine
            .valid_moves_iter(&state, player(1), PlayerColor::Blue)
            .unwrap_or_else(|error| panic!("valid move iterator should construct: {error}"))
            .nth(take_index)
            .unwrap_or_else(|| panic!("opening move index should be available"));

        let command = command_from_legal_move(
            1,
            &state,
            player(1),
            PlayerColor::Blue,
            legal_move,
        );

        let result = engine
            .apply(&state, command)
            .unwrap_or_else(|error| panic!("legal move should apply: {error}"));

        prop_assert_ne!(result.next_state.hash, original_hash);
        prop_assert_eq!(result.next_state.hash, blocus_core::compute_hash_full(&result.next_state));
    }

    #[test]
    fn first_legal_move_can_be_replayed_from_same_initial_state(
        game_id_value in 1u128..10_000u128,
    ) {
        let engine = BlocusEngine::new();

        let first_state = engine.initialize_game(two_player_config_with_game_id(game_id_value));
        let second_state = engine.initialize_game(two_player_config_with_game_id(game_id_value));

        let first_move = first_blue_legal_move(engine, &first_state);
        let second_move = first_blue_legal_move(engine, &second_state);

        prop_assert_eq!(first_move, second_move);

        let first_result = engine
            .apply(
                &first_state,
                command_from_legal_move(
                    1,
                    &first_state,
                    player(1),
                    PlayerColor::Blue,
                    first_move,
                ),
            )
            .unwrap_or_else(|error| panic!("first move should apply: {error}"));

        let second_result = engine
            .apply(
                &second_state,
                command_from_legal_move(
                    1,
                    &second_state,
                    player(1),
                    PlayerColor::Blue,
                    second_move,
                ),
            )
            .unwrap_or_else(|error| panic!("second move should apply: {error}"));

        prop_assert_eq!(&first_result.next_state, &second_result.next_state);
        prop_assert_eq!(first_result.next_state.hash, second_result.next_state.hash);
    }
}
