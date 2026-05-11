from __future__ import annotations

from typing import Any

import pytest
from conftest import FakeClassicEngine

from blocus_backend.repository import InMemoryGameRepository, OptimisticLockError
from blocus_backend.service import GameService, ProtocolError, _ai_seat_for_color


class _RacingRepository(InMemoryGameRepository):
    """Injects an OptimisticLockError on the next save_game call."""

    def __init__(self) -> None:
        super().__init__()
        self.fail_next_save: bool = False

    async def save_game(
        self,
        game_id: str,
        state_json: str,
        metadata: dict[str, Any],
        *,
        expected_version: int | None,
    ) -> None:
        if self.fail_next_save:
            self.fail_next_save = False
            raise OptimisticLockError(game_id, expected_version)
        await super().save_game(game_id, state_json, metadata, expected_version=expected_version)


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
async def test_service_missing_player_slot_raises_invalid_players() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    with pytest.raises(ProtocolError) as captured:
        await service.create_game(
            {
                "game_id": "game-incomplete",
                "mode": "four_player",
                "players": {"blue": "p1", "yellow": "p2", "red": "p3"},
            }
        )

    assert captured.value.code == "invalid_players"


@pytest.mark.asyncio
async def test_service_missing_required_field_raises_missing_field() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())
    await service.create_game(
        {
            "game_id": "game-missing",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )

    with pytest.raises(ProtocolError) as captured:
        await service.place_move(
            {
                "game_id": "game-missing",
                "command_id": "cmd",
                "player_id": "p1",
                "color": "blue",
            }
        )

    assert captured.value.code == "missing_field"


@pytest.mark.asyncio
async def test_service_invalid_scoring_raises_invalid_scoring() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    with pytest.raises(ProtocolError) as captured:
        await service.create_game(
            {
                "game_id": "game-bad-scoring",
                "mode": "two_player",
                "scoring": "weird",
                "players": {"blue_red": "p1", "yellow_green": "p2"},
            }
        )

    assert captured.value.code == "invalid_scoring"


@pytest.mark.asyncio
async def test_service_place_move_raises_conflict_on_optimistic_lock() -> None:
    repo = _RacingRepository()
    service = GameService(repo, FakeClassicEngine())
    await service.create_game(
        {
            "game_id": "game-conflict",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )
    repo.fail_next_save = True

    with pytest.raises(ProtocolError) as captured:
        await service.place_move(
            {
                "game_id": "game-conflict",
                "command_id": "cmd",
                "player_id": "p1",
                "color": "blue",
                "piece_id": 0,
                "orientation_id": 0,
                "row": 0,
                "col": 0,
            }
        )

    assert captured.value.code == "conflict"


@pytest.mark.asyncio
async def test_service_attach_ai_raises_conflict_on_optimistic_lock() -> None:
    repo = _RacingRepository()
    service = GameService(repo, FakeClassicEngine())
    await service.create_game(
        {
            "game_id": "game-attach-conflict",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )
    repo.fail_next_save = True

    with pytest.raises(ProtocolError) as captured:
        await service.attach_ai(
            {
                "game_id": "game-attach-conflict",
                "player_id": "p1",
                "color": "blue",
            }
        )

    assert captured.value.code == "conflict"


@pytest.mark.asyncio
async def test_advance_ai_turns_continues_after_conflict() -> None:
    repo = _RacingRepository()
    service = GameService(repo, FakeClassicEngine())
    await service.create_game(
        {
            "game_id": "game-ai-conflict",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )
    await service.attach_ai({"game_id": "game-ai-conflict", "player_id": "p1", "color": "blue"})

    repo.fail_next_save = True

    events = await service.advance_ai_turns("game-ai-conflict")

    assert len(events) == 1
    assert events[0]["type"] == "move_applied"


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


@pytest.mark.asyncio
async def test_service_creates_duo_game_with_default_turn_order() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    event = await service.create_game(
        {
            "game_id": "game-duo",
            "mode": "duo",
            "players": {"black": "p1", "white": "p2"},
        }
    )

    assert event["type"] == "game_created"
    assert event["state"]["mode"] == "duo"
    assert event["state"]["turn_order"] == ["black", "white"]


@pytest.mark.asyncio
async def test_service_creates_duo_game_with_first_color_white() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    event = await service.create_game(
        {
            "game_id": "game-duo-white",
            "mode": "duo",
            "first_color": "white",
            "players": {"black": "p1", "white": "p2"},
        }
    )

    assert event["state"]["turn_order"] == ["white", "black"]


@pytest.mark.asyncio
async def test_service_rejects_duo_with_basic_scoring() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    with pytest.raises(ProtocolError) as captured:
        await service.create_game(
            {
                "game_id": "game-duo-bad-scoring",
                "mode": "duo",
                "scoring": "basic",
                "players": {"black": "p1", "white": "p2"},
            }
        )

    assert captured.value.code == "invalid_scoring"


@pytest.mark.asyncio
async def test_service_rejects_duo_with_classic_first_color() -> None:
    service = GameService(InMemoryGameRepository(), FakeClassicEngine())

    with pytest.raises(ProtocolError) as captured:
        await service.create_game(
            {
                "game_id": "game-duo-bad-color",
                "mode": "duo",
                "first_color": "blue",
                "players": {"black": "p1", "white": "p2"},
            }
        )

    assert captured.value.code == "invalid_duo_color"


@pytest.mark.asyncio
async def test_advance_ai_turns_yields_when_human_seat_is_bound() -> None:
    """Binding ≻ AI: a bound human seat suspends the AI loop for that color."""
    bound_seats: set[tuple[str, str]] = set()
    service = GameService(
        InMemoryGameRepository(),
        FakeClassicEngine(),
        seat_binding_check=lambda g, p: (g, p) in bound_seats,
    )
    await service.create_game(
        {
            "game_id": "game-bind-yield",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )
    await service.attach_ai({"game_id": "game-bind-yield", "player_id": "p1", "color": "blue"})
    bound_seats.add(("game-bind-yield", "p1"))

    events = await service.advance_ai_turns("game-bind-yield")

    assert events == []


@pytest.mark.asyncio
async def test_service_maps_rule_violation_to_protocol_error() -> None:
    pytest.importorskip("blocus_engine")
    import blocus_engine as be

    class _RuleViolatingEngine(FakeClassicEngine):
        def place_move(self, _state: Any, _payload: dict[str, Any]) -> Any:
            raise be.RuleViolationError("piece overlaps existing placement")

    service = GameService(InMemoryGameRepository(), _RuleViolatingEngine())
    await service.create_game(
        {
            "game_id": "game-rule",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )

    with pytest.raises(ProtocolError) as captured:
        await service.place_move(
            {
                "game_id": "game-rule",
                "command_id": "cmd",
                "player_id": "p1",
                "color": "blue",
                "piece_id": 0,
                "orientation_id": 0,
                "row": 0,
                "col": 0,
            }
        )

    assert captured.value.code == "rule_violation"


@pytest.mark.asyncio
async def test_service_maps_input_error_to_invalid_command() -> None:
    pytest.importorskip("blocus_engine")
    import blocus_engine as be

    class _BadInputEngine(FakeClassicEngine):
        def place_move(self, _state: Any, _payload: dict[str, Any]) -> Any:
            raise be.InputError("piece_id 999 is unknown")

    service = GameService(InMemoryGameRepository(), _BadInputEngine())
    await service.create_game(
        {
            "game_id": "game-input",
            "mode": "two_player",
            "players": {"blue_red": "p1", "yellow_green": "p2"},
        }
    )

    with pytest.raises(ProtocolError) as captured:
        await service.place_move(
            {
                "game_id": "game-input",
                "command_id": "cmd",
                "player_id": "p1",
                "color": "blue",
                "piece_id": 999,
                "orientation_id": 0,
                "row": 0,
                "col": 0,
            }
        )

    assert captured.value.code == "invalid_command"
