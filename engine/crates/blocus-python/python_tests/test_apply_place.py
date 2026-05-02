from __future__ import annotations

import pytest

import blocus_engine


GAME_ID = "00000000-0000-0000-0000-000000000300"
PLAYER_1 = "00000000-0000-0000-0000-000000000001"
PLAYER_2 = "00000000-0000-0000-0000-000000000002"


def valid_uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def initialized_two_player_state() -> tuple[blocus_engine.BlocusEngine, blocus_engine.GameState]:
    engine = blocus_engine.BlocusEngine()
    config = blocus_engine.GameConfig.two_player(
        GAME_ID,
        PLAYER_1,
        PLAYER_2,
        blocus_engine.ScoringMode.BASIC,
    )

    return engine, engine.initialize_game(config)


def test_apply_opening_place_returns_game_result_and_next_state() -> None:
    engine, state = initialized_two_player_state()

    command = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_1,
        color=blocus_engine.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )

    result = engine.apply(state, command)

    assert isinstance(result, blocus_engine.GameResult)
    assert result.next_state.version == 1
    assert result.next_state.board_is_empty is False
    assert result.next_state.current_color == blocus_engine.PlayerColor.YELLOW

    assert [event.kind for event in result.events] == ["move_applied", "turn_advanced"]
    assert [event.version for event in result.events] == [1, 1]
    assert result.response.kind == "move_applied"
    assert result.response.message == "move applied"

    assert state.version == 0
    assert state.board_is_empty is True
    assert state.current_color == blocus_engine.PlayerColor.BLUE


def test_apply_opening_place_maps_rule_violation() -> None:
    engine, state = initialized_two_player_state()

    command = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_1,
        color=blocus_engine.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=1,
    )

    with pytest.raises(blocus_engine.RuleViolationError) as captured:
        engine.apply(state, command)

    assert "rule_violation" in str(captured.value)
    assert "MissingCornerContact" in str(captured.value)


def test_apply_wrong_turn_maps_rule_violation() -> None:
    engine, state = initialized_two_player_state()

    command = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_2,
        color=blocus_engine.PlayerColor.YELLOW,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=19,
    )

    with pytest.raises(blocus_engine.RuleViolationError) as captured:
        engine.apply(state, command)

    assert "rule_violation" in str(captured.value)
    assert "WrongPlayerTurn" in str(captured.value)


def test_apply_game_id_mismatch_maps_input_error() -> None:
    engine, state = initialized_two_player_state()

    command = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(999),
        player_id=PLAYER_1,
        color=blocus_engine.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )

    with pytest.raises(blocus_engine.InputError) as captured:
        engine.apply(state, command)

    assert "input_error" in str(captured.value)
    assert "GameIdMismatch" in str(captured.value)