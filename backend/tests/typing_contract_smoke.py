from __future__ import annotations

import blocus_engine as be


def use_contract() -> None:
    assert isinstance(be.engine_health(), bool)

    color: be.PlayerColor = be.PlayerColor.BLUE
    status: be.GameStatus = be.GameStatus.IN_PROGRESS
    scoring: be.ScoringMode = be.ScoringMode.BASIC

    assert color.value == "blue"
    assert status.value == "in_progress"
    assert scoring.value == "basic"

    place = be.PlaceCommand(
        command_id="00000000-0000-0000-0000-000000000001",
        game_id="00000000-0000-0000-0000-000000000002",
        player_id="00000000-0000-0000-0000-000000000003",
        color=be.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )

    assert place.color == be.PlayerColor.BLUE
    assert place.piece_id == 0
    assert place.orientation_id == 0
    assert place.row == 0
    assert place.col == 0
    assert place.board_index == 0

    passed = be.PassCommand(
        command_id="00000000-0000-0000-0000-000000000004",
        game_id=place.game_id,
        player_id=place.player_id,
        color=be.PlayerColor.BLUE,
    )

    assert passed.game_id == place.game_id
