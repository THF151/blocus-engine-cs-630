from __future__ import annotations

import pytest

import blocus_engine


def valid_uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def test_place_command_converts_row_col_to_board_index() -> None:
    command = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.BLUE,
        piece_id=4,
        orientation_id=5,
        row=6,
        col=7,
    )

    assert command.command_id == valid_uuid(1)
    assert command.game_id == valid_uuid(2)
    assert command.player_id == valid_uuid(3)
    assert command.color == blocus_engine.PlayerColor.BLUE
    assert command.piece_id == 4
    assert command.orientation_id == 5
    assert command.row == 6
    assert command.col == 7
    assert command.board_index == 199
    assert "PlaceCommand" in repr(command)


def test_place_command_equality_is_value_based() -> None:
    first = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )
    duplicate = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )
    different = blocus_engine.PlaceCommand(
        command_id=valid_uuid(4),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )

    assert first == duplicate
    assert first != different


def test_pass_command_preserves_typed_fields() -> None:
    command = blocus_engine.PassCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.YELLOW,
    )

    assert command.command_id == valid_uuid(1)
    assert command.game_id == valid_uuid(2)
    assert command.player_id == valid_uuid(3)
    assert command.color == blocus_engine.PlayerColor.YELLOW
    assert "PassCommand" in repr(command)


def test_pass_command_equality_is_value_based() -> None:
    first = blocus_engine.PassCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.YELLOW,
    )
    duplicate = blocus_engine.PassCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.YELLOW,
    )
    different = blocus_engine.PassCommand(
        command_id=valid_uuid(4),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.YELLOW,
    )

    assert first == duplicate
    assert first != different


@pytest.mark.parametrize(
    ("row", "col", "expected_board_index"),
    [
        (0, 0, 0),
        (0, 19, 19),
        (1, 0, 32),
        (19, 19, 627),
    ],
)
def test_place_command_uses_padded_row_board_indexing(
    row: int,
    col: int,
    expected_board_index: int,
) -> None:
    command = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.RED,
        piece_id=0,
        orientation_id=0,
        row=row,
        col=col,
    )

    assert command.board_index == expected_board_index
