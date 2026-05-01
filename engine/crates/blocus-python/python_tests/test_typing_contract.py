from __future__ import annotations

import uuid
from typing import get_type_hints

import blocus_engine


def valid_uuid(value: int) -> str:
    return str(uuid.UUID(int=value))


def accepts_color(color: blocus_engine.PlayerColor) -> str:
    return color.value


def accepts_status(status: blocus_engine.GameStatus) -> str:
    return status.value


def accepts_scoring_mode(scoring: blocus_engine.ScoringMode) -> str:
    return scoring.value


def accepts_place_command(command: blocus_engine.PlaceCommand) -> int:
    return command.board_index


def accepts_pass_command(command: blocus_engine.PassCommand) -> str:
    return command.color.value


def test_runtime_type_annotations_resolve_to_extension_classes() -> None:
    assert get_type_hints(accepts_color)["color"] is blocus_engine.PlayerColor
    assert get_type_hints(accepts_status)["status"] is blocus_engine.GameStatus
    assert get_type_hints(accepts_scoring_mode)["scoring"] is blocus_engine.ScoringMode
    assert get_type_hints(accepts_place_command)["command"] is blocus_engine.PlaceCommand
    assert get_type_hints(accepts_pass_command)["command"] is blocus_engine.PassCommand


def test_typed_helper_functions_accept_public_python_contract_objects() -> None:
    place_command = blocus_engine.PlaceCommand(
        command_id=valid_uuid(1),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.GREEN,
        piece_id=0,
        orientation_id=0,
        row=1,
        col=2,
    )
    pass_command = blocus_engine.PassCommand(
        command_id=valid_uuid(4),
        game_id=valid_uuid(2),
        player_id=valid_uuid(3),
        color=blocus_engine.PlayerColor.BLUE,
    )

    assert accepts_color(blocus_engine.PlayerColor.BLUE) == "blue"
    assert accepts_status(blocus_engine.GameStatus.IN_PROGRESS) == "in_progress"
    assert accepts_scoring_mode(blocus_engine.ScoringMode.ADVANCED) == "advanced"
    assert accepts_place_command(place_command) == 34
    assert accepts_pass_command(pass_command) == "blue"
