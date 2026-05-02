from __future__ import annotations

import pytest

import blocus_engine


GAME_ID = "00000000-0000-0000-0000-000000000600"
PLAYER_1 = "00000000-0000-0000-0000-000000000001"
PLAYER_2 = "00000000-0000-0000-0000-000000000002"


def initialized_two_player_state() -> tuple[blocus_engine.BlocusEngine, blocus_engine.GameState]:
    engine = blocus_engine.BlocusEngine()
    config = blocus_engine.GameConfig.two_player(
        GAME_ID,
        PLAYER_1,
        PLAYER_2,
        blocus_engine.ScoringMode.BASIC,
    )

    return engine, engine.initialize_game(config)


def test_score_game_rejects_unfinished_game_with_structured_rule_violation() -> None:
    engine, state = initialized_two_player_state()

    with pytest.raises(blocus_engine.RuleViolationError) as captured:
        engine.score_game(state, blocus_engine.ScoringMode.BASIC)

    assert "rule_violation" in str(captured.value)
    assert "GameNotFinished" in str(captured.value)