from __future__ import annotations

import pytest

import blocus_engine


def valid_uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def assert_structured_input_error(error: BaseException, expected_code: str) -> None:
    text = str(error)
    assert "input_error" in text
    assert expected_code in text


def test_error_classes_exist_and_are_exceptions() -> None:
    assert issubclass(blocus_engine.BlocusError, Exception)
    assert issubclass(blocus_engine.InputError, blocus_engine.BlocusError)
    assert issubclass(blocus_engine.RuleViolationError, blocus_engine.BlocusError)
    assert issubclass(blocus_engine.EngineError, blocus_engine.BlocusError)


def test_invalid_color_raises_structured_input_error() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PlayerColor("purple")

    assert_structured_input_error(captured.value, "InvalidGameConfig")


def test_invalid_uuid_raises_structured_input_error() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PassCommand(
            command_id="not-a-uuid",
            game_id=valid_uuid(2),
            player_id=valid_uuid(3),
            color=blocus_engine.PlayerColor.BLUE,
        )

    assert_structured_input_error(captured.value, "InvalidGameConfig")


def test_invalid_piece_id_raises_structured_input_error() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PlaceCommand(
            command_id=valid_uuid(1),
            game_id=valid_uuid(2),
            player_id=valid_uuid(3),
            color=blocus_engine.PlayerColor.BLUE,
            piece_id=21,
            orientation_id=0,
            row=0,
            col=0,
        )

    assert_structured_input_error(captured.value, "UnknownPiece")


def test_invalid_orientation_id_raises_structured_input_error() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PlaceCommand(
            command_id=valid_uuid(1),
            game_id=valid_uuid(2),
            player_id=valid_uuid(3),
            color=blocus_engine.PlayerColor.BLUE,
            piece_id=0,
            orientation_id=8,
            row=0,
            col=0,
        )

    assert_structured_input_error(captured.value, "UnknownOrientation")


def test_invalid_row_col_raises_structured_input_error() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PlaceCommand(
            command_id=valid_uuid(1),
            game_id=valid_uuid(2),
            player_id=valid_uuid(3),
            color=blocus_engine.PlayerColor.BLUE,
            piece_id=0,
            orientation_id=0,
            row=20,
            col=0,
        )

    assert_structured_input_error(captured.value, "InvalidBoardIndex")


def test_unimplemented_engine_methods_raise_structured_engine_error() -> None:
    engine = blocus_engine.BlocusEngine()
    config = blocus_engine.GameConfig.two_player(
        valid_uuid(100),
        valid_uuid(1),
        valid_uuid(2),
        blocus_engine.ScoringMode.BASIC,
    )
    state = engine.initialize_game(config)

    with pytest.raises(blocus_engine.EngineError) as captured:
        engine.has_any_valid_move(state, valid_uuid(1), blocus_engine.PlayerColor.BLUE)

    text = str(captured.value)
    assert "engine_error" in text
    assert "InvariantViolation" in text
