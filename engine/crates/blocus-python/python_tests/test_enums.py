from __future__ import annotations

import pytest

import blocus_engine


def test_player_color_constants_are_stable() -> None:
    cases = [
        (blocus_engine.PlayerColor.BLUE, "BLUE", "blue", 0),
        (blocus_engine.PlayerColor.YELLOW, "YELLOW", "yellow", 1),
        (blocus_engine.PlayerColor.RED, "RED", "red", 2),
        (blocus_engine.PlayerColor.GREEN, "GREEN", "green", 3),
    ]

    for color, name, value, index in cases:
        assert color.name == name
        assert color.value == value
        assert color.index == index
        assert str(color) == value
        assert repr(color) == f"PlayerColor.{name}"
        assert blocus_engine.PlayerColor(value) == color


def test_player_color_equality_and_hashing_are_value_based() -> None:
    assert blocus_engine.PlayerColor.BLUE == blocus_engine.PlayerColor("blue")
    assert blocus_engine.PlayerColor.BLUE != blocus_engine.PlayerColor.YELLOW

    colors = {
        blocus_engine.PlayerColor.BLUE,
        blocus_engine.PlayerColor("blue"),
        blocus_engine.PlayerColor.YELLOW,
    }

    assert len(colors) == 2
    assert blocus_engine.PlayerColor.BLUE in colors
    assert blocus_engine.PlayerColor.YELLOW in colors


def test_game_status_constants_are_stable() -> None:
    cases = [
        (blocus_engine.GameStatus.IN_PROGRESS, "IN_PROGRESS", "in_progress"),
        (blocus_engine.GameStatus.FINISHED, "FINISHED", "finished"),
    ]

    for status, name, value in cases:
        assert status.name == name
        assert status.value == value
        assert str(status) == value
        assert repr(status) == f"GameStatus.{name}"
        assert blocus_engine.GameStatus(value) == status


def test_scoring_mode_constants_are_stable() -> None:
    cases = [
        (blocus_engine.ScoringMode.BASIC, "BASIC", "basic"),
        (blocus_engine.ScoringMode.ADVANCED, "ADVANCED", "advanced"),
    ]

    for scoring, name, value in cases:
        assert scoring.name == name
        assert scoring.value == value
        assert str(scoring) == value
        assert repr(scoring) == f"ScoringMode.{name}"
        assert blocus_engine.ScoringMode(value) == scoring


def test_invalid_game_status_and_scoring_mode_raise_input_error() -> None:
    with pytest.raises(blocus_engine.InputError):
        blocus_engine.GameStatus("paused")

    with pytest.raises(blocus_engine.InputError):
        blocus_engine.ScoringMode("expert")
