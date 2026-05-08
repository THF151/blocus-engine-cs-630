from __future__ import annotations

import pytest
from conftest import FakeClassicEngine

from blocus_backend.repository import InMemoryGameRepository
from blocus_backend.service import GameService, ProtocolError, _ai_seat_for_color


def test_ai_seat_for_three_player_shared_green_uses_scheduled_player() -> None:
    metadata = {
        "mode": "three_player",
        "players": {"shared_green": ["p1", "p2", "p3"]},
        "ai_seats": [
            {"player_id": "p1", "color": "green"},
            {"player_id": "p2", "color": "green"},
            {"player_id": "p3", "color": "green"},
        ],
    }
    state_json = '{"turn": {"shared_color_turn_index": 1}}'

    seat = _ai_seat_for_color(metadata, "green", state_json)

    assert seat == {"player_id": "p2", "color": "green"}


@pytest.mark.asyncio
async def test_service_rejects_non_classic_color_in_place_move() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())
    await service.create_game(
        {
            "game_id": "game-color",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )

    with pytest.raises(ProtocolError) as captured:
        await service.place_move(
            {
                "game_id": "game-color",
                "command_id": "cmd",
                "player_id": "p1",
                "color": "black",
                "piece_id": 0,
                "orientation_id": 0,
                "row": 0,
                "col": 0,
            }
        )

    assert captured.value.code == "invalid_classic_color"


@pytest.mark.asyncio
async def test_service_returns_legal_moves_and_score_report() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())
    await service.create_game(
        {
            "game_id": "game-flow",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )

    legal_moves = await service.legal_moves(
        {"game_id": "game-flow", "player_id": "p1", "color": "blue"}
    )
    score = await service.score("game-flow")

    assert legal_moves["type"] == "legal_moves"
    assert legal_moves["moves"][0]["piece_id"] == 0
    assert score == {
        "type": "score_report",
        "game_id": "game-flow",
        "score": {"scoring": "basic", "entries": []},
    }


@pytest.mark.asyncio
async def test_service_pass_move_persists_and_reports_pass_event() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())
    await service.create_game(
        {
            "game_id": "game-pass",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )

    event = await service.pass_move(
        {
            "game_id": "game-pass",
            "command_id": "cmd-pass",
            "player_id": "p1",
            "color": "blue",
        }
    )
    snapshot = await service.state_snapshot("game-pass")

    assert event["type"] == "pass_applied"
    assert event["state"]["version"] == 1
    assert snapshot["state"]["version"] == 1


@pytest.mark.asyncio
async def test_service_reports_missing_game() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    with pytest.raises(ProtocolError) as captured:
        await service.state_snapshot("missing")

    assert captured.value.code == "game_not_found"


@pytest.mark.asyncio
async def test_service_validates_four_player_start_color() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    with pytest.raises(ProtocolError) as captured:
        await service.create_game(
            {
                "game_id": "game-invalid-start",
                "mode": "four_player",
                "first_color": "black",
                "players": {
                    "blue": "p1",
                    "yellow": "p2",
                    "red": "p3",
                    "green": "p4",
                },
            }
        )

    assert captured.value.code == "invalid_classic_color"
