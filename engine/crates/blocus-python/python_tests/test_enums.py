from __future__ import annotations

import pytest

import blocus_engine


def test_player_color_constants_are_stable() -> None:
    cases = [
        (blocus_engine.PlayerColor.BLUE, "BLUE", "blue", 0, "PlayerColor.BLUE"),
        (blocus_engine.PlayerColor.YELLOW, "YELLOW", "yellow", 1, "PlayerColor.YELLOW"),
        (blocus_engine.PlayerColor.RED, "RED", "red", 2, "PlayerColor.RED"),
        (blocus_engine.PlayerColor.GREEN, "GREEN", "green", 3, "PlayerColor.GREEN"),
    ]

    for color, expected_name, expected_value, expected_index, expected_repr in cases:
        assert color.name == expected_name
        assert color.value == expected_value
        assert color.index == expected_index
        assert str(color) == expected_value
        assert repr(color) == expected_repr


def test_player_color_factories_and_constructor_are_value_based() -> None:
    assert blocus_engine.PlayerColor.BLUE == blocus_engine.PlayerColor.blue()
    assert blocus_engine.PlayerColor.YELLOW == blocus_engine.PlayerColor.yellow()
    assert blocus_engine.PlayerColor.RED == blocus_engine.PlayerColor.red()
    assert blocus_engine.PlayerColor.GREEN == blocus_engine.PlayerColor.green()

    assert blocus_engine.PlayerColor.BLUE == blocus_engine.PlayerColor("blue")
    assert blocus_engine.PlayerColor.BLUE == blocus_engine.PlayerColor("BLUE")
    assert blocus_engine.PlayerColor.BLUE != blocus_engine.PlayerColor.YELLOW

    assert len({blocus_engine.PlayerColor.BLUE, blocus_engine.PlayerColor("blue")}) == 1


def test_game_status_constants_are_stable() -> None:
    cases = [
        (blocus_engine.GameStatus.IN_PROGRESS, "IN_PROGRESS", "in_progress"),
        (blocus_engine.GameStatus.FINISHED, "FINISHED", "finished"),
    ]

    for status, expected_name, expected_value in cases:
        assert status.name == expected_name
        assert status.value == expected_value
        assert str(status) == expected_value
        assert repr(status) == f"GameStatus.{expected_name}"


def test_game_status_factories_and_constructor_are_value_based() -> None:
    assert blocus_engine.GameStatus.IN_PROGRESS == blocus_engine.GameStatus.in_progress()
    assert blocus_engine.GameStatus.FINISHED == blocus_engine.GameStatus.finished()
    assert blocus_engine.GameStatus.IN_PROGRESS == blocus_engine.GameStatus("in_progress")
    assert blocus_engine.GameStatus.FINISHED == blocus_engine.GameStatus("FINISHED")
    assert blocus_engine.GameStatus.IN_PROGRESS != blocus_engine.GameStatus.FINISHED


def test_scoring_mode_constants_are_stable() -> None:
    cases = [
        (blocus_engine.ScoringMode.BASIC, "BASIC", "basic"),
        (blocus_engine.ScoringMode.ADVANCED, "ADVANCED", "advanced"),
    ]

    for scoring, expected_name, expected_value in cases:
        assert scoring.name == expected_name
        assert scoring.value == expected_value
        assert str(scoring) == expected_value
        assert repr(scoring) == f"ScoringMode.{expected_name}"


def test_scoring_mode_factories_and_constructor_are_value_based() -> None:
    assert blocus_engine.ScoringMode.BASIC == blocus_engine.ScoringMode.basic()
    assert blocus_engine.ScoringMode.ADVANCED == blocus_engine.ScoringMode.advanced()
    assert blocus_engine.ScoringMode.BASIC == blocus_engine.ScoringMode("basic")
    assert blocus_engine.ScoringMode.ADVANCED == blocus_engine.ScoringMode("ADVANCED")
    assert blocus_engine.ScoringMode.BASIC != blocus_engine.ScoringMode.ADVANCED


def test_game_mode_constants_are_stable() -> None:
    cases = [
        (blocus_engine.GameMode.TWO_PLAYER, "TWO_PLAYER", "two_player"),
        (blocus_engine.GameMode.THREE_PLAYER, "THREE_PLAYER", "three_player"),
        (blocus_engine.GameMode.FOUR_PLAYER, "FOUR_PLAYER", "four_player"),
    ]

    for mode, expected_name, expected_value in cases:
        assert mode.name == expected_name
        assert mode.value == expected_value
        assert str(mode) == expected_value
        assert repr(mode) == f"GameMode.{expected_name}"


def test_game_mode_factories_and_constructor_are_value_based() -> None:
    assert blocus_engine.GameMode.TWO_PLAYER == blocus_engine.GameMode.two_player()
    assert blocus_engine.GameMode.THREE_PLAYER == blocus_engine.GameMode.three_player()
    assert blocus_engine.GameMode.FOUR_PLAYER == blocus_engine.GameMode.four_player()

    assert blocus_engine.GameMode.TWO_PLAYER == blocus_engine.GameMode("two_player")
    assert blocus_engine.GameMode.THREE_PLAYER == blocus_engine.GameMode("THREE_PLAYER")
    assert blocus_engine.GameMode.FOUR_PLAYER == blocus_engine.GameMode("FourPlayer")


@pytest.mark.parametrize(
    ("constructor", "value"),
    [
        (blocus_engine.PlayerColor, "purple"),
        (blocus_engine.GameStatus, "paused"),
        (blocus_engine.ScoringMode, "bonus"),
        (blocus_engine.GameMode, "solo"),
    ],
)
def test_invalid_enum_values_raise_structured_input_error(constructor: object, value: str) -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        constructor(value)  # type: ignore[operator]

    assert "input_error" in str(captured.value)
    assert "InvalidGameConfig" in str(captured.value)
