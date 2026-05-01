from __future__ import annotations

import uuid

import pytest

import blocus_engine


def valid_uuid(value: int) -> str:
    return str(uuid.UUID(int=value))


def assert_blocus_error(
    error: BaseException,
    *,
    expected_type: type[BaseException],
    code: str,
    category: str,
    message: str,
) -> None:
    assert isinstance(error, blocus_engine.BlocusError)
    assert isinstance(error, expected_type)
    assert getattr(error, "code") == code
    assert getattr(error, "category") == category
    assert getattr(error, "message") == message
    assert str(error) == message


def test_exception_classes_are_exposed_with_expected_hierarchy() -> None:
    assert issubclass(blocus_engine.RuleViolationError, blocus_engine.BlocusError)
    assert issubclass(blocus_engine.InputError, blocus_engine.BlocusError)
    assert issubclass(blocus_engine.EngineError, blocus_engine.BlocusError)
    assert issubclass(blocus_engine.BlocusError, Exception)


def test_invalid_color_raises_structured_input_error() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PlayerColor("purple")

    assert_blocus_error(
        captured.value,
        expected_type=blocus_engine.InputError,
        code="InvalidGameConfig",
        category="input_error",
        message="invalid game configuration",
    )


def test_invalid_uuid_raises_structured_input_error() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PassCommand(
            command_id="not-a-uuid",
            game_id=valid_uuid(2),
            player_id=valid_uuid(3),
            color=blocus_engine.PlayerColor.BLUE,
        )

    assert_blocus_error(
        captured.value,
        expected_type=blocus_engine.InputError,
        code="InvalidGameConfig",
        category="input_error",
        message="invalid game configuration",
    )


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

    assert_blocus_error(
        captured.value,
        expected_type=blocus_engine.InputError,
        code="UnknownPiece",
        category="input_error",
        message="unknown piece",
    )


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

    assert_blocus_error(
        captured.value,
        expected_type=blocus_engine.InputError,
        code="UnknownOrientation",
        category="input_error",
        message="unknown orientation",
    )


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

    assert_blocus_error(
        captured.value,
        expected_type=blocus_engine.InputError,
        code="InvalidBoardIndex",
        category="input_error",
        message="invalid board index",
    )
