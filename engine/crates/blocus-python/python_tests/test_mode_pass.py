from __future__ import annotations

import pytest

import blocus_engine


GAME_ID = "00000000-0000-0000-0000-000000000500"
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


def test_get_valid_moves_returns_legal_move_wrappers() -> None:
    engine, state = initialized_two_player_state()

    moves = engine.get_valid_moves(state, PLAYER_1, blocus_engine.PlayerColor.BLUE)

    assert moves
    assert isinstance(moves[0], blocus_engine.LegalMove)
    assert moves[0].piece_id == 0
    assert moves[0].orientation_id == 0
    assert moves[0].row == 0
    assert moves[0].col == 0
    assert moves[0].board_index == 0
    assert moves[0].score_delta == 1
    assert "LegalMove" in repr(moves[0])


def test_get_valid_moves_returns_empty_for_wrong_turn() -> None:
    engine, state = initialized_two_player_state()

    moves = engine.get_valid_moves(state, PLAYER_2, blocus_engine.PlayerColor.YELLOW)

    assert moves == []


def test_pass_rejected_when_current_color_has_legal_move() -> None:
    engine, state = initialized_two_player_state()

    command = blocus_engine.PassCommand(
        command_id=valid_uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_1,
        color=blocus_engine.PlayerColor.BLUE,
    )

    with pytest.raises(blocus_engine.RuleViolationError) as captured:
        engine.apply(state, command)

    assert "rule_violation" in str(captured.value)
    assert "PassNotAllowedBecauseMoveExists" in str(captured.value)