from __future__ import annotations

import blocus_engine


def valid_uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def test_typed_initialization_contract_runtime_smoke() -> None:
    engine: blocus_engine.BlocusEngine = blocus_engine.BlocusEngine()
    scoring: blocus_engine.ScoringMode = blocus_engine.ScoringMode.BASIC
    config: blocus_engine.GameConfig = blocus_engine.GameConfig.two_player(
        valid_uuid(200),
        valid_uuid(1),
        valid_uuid(2),
        scoring,
    )

    state: blocus_engine.GameState = engine.initialize_game(config)

    status: blocus_engine.GameStatus = state.status
    mode: blocus_engine.GameMode = state.mode
    color: blocus_engine.PlayerColor = state.current_color
    board_is_empty: bool = state.board_is_empty
    version: int = state.version

    assert status == blocus_engine.GameStatus.IN_PROGRESS
    assert mode == blocus_engine.GameMode.TWO_PLAYER
    assert color == blocus_engine.PlayerColor.BLUE
    assert board_is_empty is True
    assert version == 0


def test_typed_command_contract_runtime_smoke() -> None:
    place_command: blocus_engine.PlaceCommand = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.GREEN,
        piece_id=0,
        orientation_id=0,
        row=1,
        col=2,
    )
    pass_command: blocus_engine.PassCommand = blocus_engine.PassCommand(
        command_id=valid_uuid(4),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.GREEN,
    )

    assert place_command.color == blocus_engine.PlayerColor.GREEN
    assert place_command.board_index == 34
    assert pass_command.color == blocus_engine.PlayerColor.GREEN
