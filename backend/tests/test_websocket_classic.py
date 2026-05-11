from __future__ import annotations

from typing import Any

from conftest import FakeClassicEngine, FakeState
from fastapi.testclient import TestClient

from blocus_backend.app import create_app
from blocus_backend.event_bus import InMemoryGameEventBus
from blocus_backend.repository import InMemoryGameRepository


def make_client() -> TestClient:
    app = create_app(
        repository=InMemoryGameRepository(),
        engine=FakeClassicEngine(),
        event_bus=InMemoryGameEventBus(),
    )
    return TestClient(app)


class ExplodingEngine(FakeClassicEngine):
    def state_view(self, _state: FakeState) -> dict[str, Any]:
        raise RuntimeError("redis://secret-host:6379 traceback fragment")


class _ClosableBus(InMemoryGameEventBus):
    def __init__(self) -> None:
        super().__init__()
        self.closed = False

    async def aclose(self) -> None:
        self.closed = True


class _ClosableRepository(InMemoryGameRepository):
    def __init__(self) -> None:
        super().__init__()
        self.closed = False

    async def aclose(self) -> None:
        self.closed = True


def test_websocket_creates_classic_four_player_game_with_rotated_turn_order() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-1",
                    "mode": "four_player",
                    "scoring": "basic",
                    "first_color": "red",
                    "players": {
                        "blue": "player-blue",
                        "yellow": "player-yellow",
                        "red": "player-red",
                        "green": "player-green",
                    },
                },
            }
        )

        event = ws.receive_json()

    assert event["type"] == "game_created"
    assert event["game_id"] == "game-1"
    assert event["state"]["mode"] == "four_player"
    assert event["state"]["turn_order"] == ["red", "green", "blue", "yellow"]


def test_websocket_creates_duo_game() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-duo",
                    "mode": "duo",
                    "players": {"black": "p1", "white": "p2"},
                },
            }
        )

        event = ws.receive_json()

    assert event["type"] == "game_created"
    assert event["state"]["mode"] == "duo"
    assert event["state"]["turn_order"] == ["black", "white"]


def test_websocket_rejects_unknown_mode() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-bad",
                    "mode": "five_player",
                    "players": {"blue": "p1"},
                },
            }
        )

        event = ws.receive_json()

    assert event["type"] == "error"
    assert event["code"] == "invalid_mode"


def test_move_application_is_broadcast_to_subscribed_clients() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as owner:
        owner.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-broadcast",
                    "mode": "two_player",
                    "scoring": "basic",
                    "players": {
                        "blue_red": "player-one",
                        "yellow_green": "player-two",
                    },
                },
            }
        )
        owner.receive_json()
        owner.send_json(
            {
                "action": "subscribe_game",
                "payload": {"game_id": "game-broadcast", "player_id": "player-one"},
            }
        )
        assert owner.receive_json()["type"] == "state_snapshot"

        with client.websocket_connect("/ws") as subscriber:
            subscriber.send_json(
                {
                    "action": "subscribe_game",
                    "payload": {"game_id": "game-broadcast"},
                }
            )
            assert subscriber.receive_json()["type"] == "state_snapshot"

            owner.send_json(
                {
                    "action": "place_move",
                    "payload": {
                        "game_id": "game-broadcast",
                        "command_id": "cmd-1",
                        "player_id": "player-one",
                        "color": "blue",
                        "piece_id": 0,
                        "orientation_id": 0,
                        "row": 0,
                        "col": 0,
                    },
                }
            )

            owner_event = owner.receive_json()
            subscriber_event = subscriber.receive_json()

    assert owner_event["type"] == "move_applied"
    assert owner_event["state"]["version"] == 1
    assert subscriber_event == owner_event


def test_move_application_is_broadcast_across_connection_managers() -> None:
    repository = InMemoryGameRepository()
    event_bus = InMemoryGameEventBus()
    app_one = create_app(repository=repository, engine=FakeClassicEngine(), event_bus=event_bus)
    app_two = create_app(repository=repository, engine=FakeClassicEngine(), event_bus=event_bus)
    owner_client = TestClient(app_one)
    subscriber_client = TestClient(app_two)

    with owner_client.websocket_connect("/ws") as owner:
        owner.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-cross-worker",
                    "mode": "two_player",
                    "scoring": "basic",
                    "players": {
                        "blue_red": "player-one",
                        "yellow_green": "player-two",
                    },
                },
            }
        )
        owner.receive_json()
        owner.send_json(
            {
                "action": "subscribe_game",
                "payload": {"game_id": "game-cross-worker", "player_id": "player-one"},
            }
        )
        assert owner.receive_json()["type"] == "state_snapshot"

        with subscriber_client.websocket_connect("/ws") as subscriber:
            subscriber.send_json(
                {
                    "action": "subscribe_game",
                    "payload": {"game_id": "game-cross-worker"},
                }
            )
            assert subscriber.receive_json()["type"] == "state_snapshot"

            owner.send_json(
                {
                    "action": "place_move",
                    "payload": {
                        "game_id": "game-cross-worker",
                        "command_id": "cmd-1",
                        "player_id": "player-one",
                        "color": "blue",
                        "piece_id": 0,
                        "orientation_id": 0,
                        "row": 0,
                        "col": 0,
                    },
                }
            )

            owner_event = owner.receive_json()
            subscriber_event = subscriber.receive_json()

    assert owner_event["type"] == "move_applied"
    assert subscriber_event == owner_event
    assert event_bus.published_events[-1][0] == "game-cross-worker"


def test_attach_ai_plays_first_legal_move_for_current_turn() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-ai",
                    "mode": "two_player",
                    "scoring": "basic",
                    "players": {
                        "blue_red": "player-one",
                        "yellow_green": "player-two",
                    },
                },
            }
        )
        ws.receive_json()

        ws.send_json(
            {
                "action": "attach_ai",
                "payload": {
                    "game_id": "game-ai",
                    "player_id": "player-one",
                    "color": "blue",
                },
            }
        )

        attach_event = ws.receive_json()
        ai_event = ws.receive_json()

    assert attach_event["type"] == "game_joined"
    assert ai_event["type"] == "move_applied"
    assert ai_event["state"]["version"] == 1


def test_ai_turns_continue_until_next_non_ai_seat() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-ai-chain",
                    "mode": "two_player",
                    "scoring": "basic",
                    "players": {
                        "blue_red": "player-one",
                        "yellow_green": "player-two",
                    },
                },
            }
        )
        ws.receive_json()

        ws.send_json(
            {
                "action": "attach_ai",
                "payload": {
                    "game_id": "game-ai-chain",
                    "player_id": "player-one",
                    "color": "red",
                },
            }
        )
        assert ws.receive_json()["type"] == "game_joined"

        ws.send_json(
            {
                "action": "attach_ai",
                "payload": {
                    "game_id": "game-ai-chain",
                    "player_id": "player-one",
                    "color": "blue",
                },
            }
        )
        assert ws.receive_json()["type"] == "game_joined"
        assert ws.receive_json()["state"]["current_color"] == "yellow"

        ws.send_json(
            {
                "action": "attach_ai",
                "payload": {
                    "game_id": "game-ai-chain",
                    "player_id": "player-two",
                    "color": "yellow",
                },
            }
        )
        assert ws.receive_json()["type"] == "game_joined"
        yellow_event = ws.receive_json()
        red_event = ws.receive_json()

    assert yellow_event["type"] == "move_applied"
    assert yellow_event["state"]["current_color"] == "red"
    assert red_event["type"] == "move_applied"
    assert red_event["state"]["current_color"] == "green"


def test_websocket_unknown_action_returns_protocol_error() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json({"action": "not_real", "payload": {}})

        event = ws.receive_json()

    assert event["type"] == "error"
    assert event["code"] == "unknown_action"


def test_unhandled_engine_error_returns_opaque_internal_error() -> None:
    app = create_app(
        repository=InMemoryGameRepository(),
        engine=ExplodingEngine(),
        event_bus=InMemoryGameEventBus(),
    )
    client = TestClient(app)

    with client.websocket_connect("/ws") as ws:
        ws.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-boom",
                    "mode": "two_player",
                    "players": {"blue_red": "p1", "yellow_green": "p2"},
                },
            }
        )
        event = ws.receive_json()

    assert event["type"] == "error"
    assert event["code"] == "internal_error"
    assert event["message"] == "internal error"
    assert "secret-host" not in event["message"]


def test_non_dict_message_returns_invalid_message_error() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json([1, 2, 3])
        event = ws.receive_json()

    assert event["type"] == "error"
    assert event["code"] == "invalid_message"


def test_lifespan_closes_event_bus_and_repository() -> None:
    bus = _ClosableBus()
    repository = _ClosableRepository()
    app = create_app(repository=repository, engine=FakeClassicEngine(), event_bus=bus)

    with TestClient(app):
        pass

    assert bus.closed
    assert repository.closed


def test_websocket_can_request_state_legal_moves_and_score() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        ws.send_json(
            {
                "action": "create_game",
                "payload": {
                    "game_id": "game-queries",
                    "mode": "two_player",
                    "players": {"blue_red": "p1", "yellow_green": "p2"},
                },
            }
        )
        ws.receive_json()

        ws.send_json({"action": "request_state", "payload": {"game_id": "game-queries"}})
        state = ws.receive_json()

        ws.send_json(
            {
                "action": "request_legal_moves",
                "payload": {"game_id": "game-queries", "player_id": "p1", "color": "blue"},
            }
        )
        moves = ws.receive_json()

        ws.send_json({"action": "request_score", "payload": {"game_id": "game-queries"}})
        score = ws.receive_json()

    assert state["type"] == "state_snapshot"
    assert moves["type"] == "legal_moves"
    assert moves["moves"][0]["piece_id"] == 0
    assert score["type"] == "score_report"


def _create_two_player_game(ws: Any, game_id: str = "game-bind") -> None:
    ws.send_json(
        {
            "action": "create_game",
            "payload": {
                "game_id": game_id,
                "mode": "two_player",
                "players": {"blue_red": "player-one", "yellow_green": "player-two"},
            },
        }
    )
    ws.receive_json()


def test_place_move_without_seat_claim_returns_not_seated() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        _create_two_player_game(ws, "game-no-seat")

        ws.send_json(
            {
                "action": "place_move",
                "payload": {
                    "game_id": "game-no-seat",
                    "command_id": "cmd-1",
                    "player_id": "player-one",
                    "color": "blue",
                    "piece_id": 0,
                    "orientation_id": 0,
                    "row": 0,
                    "col": 0,
                },
            }
        )
        event = ws.receive_json()

    assert event["type"] == "error"
    assert event["code"] == "not_seated"


def test_place_move_with_mismatched_player_id_returns_player_mismatch() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        _create_two_player_game(ws, "game-mismatch")
        ws.send_json(
            {
                "action": "subscribe_game",
                "payload": {"game_id": "game-mismatch", "player_id": "player-one"},
            }
        )
        assert ws.receive_json()["type"] == "state_snapshot"

        ws.send_json(
            {
                "action": "place_move",
                "payload": {
                    "game_id": "game-mismatch",
                    "command_id": "cmd-1",
                    "player_id": "player-two",
                    "color": "yellow",
                    "piece_id": 0,
                    "orientation_id": 0,
                    "row": 0,
                    "col": 0,
                },
            }
        )
        event = ws.receive_json()

    assert event["type"] == "error"
    assert event["code"] == "player_mismatch"


def test_subscribe_without_player_id_is_spectator_mode() -> None:
    """Spectator can receive state and legal_moves but not place moves."""
    client = make_client()

    with client.websocket_connect("/ws") as ws:
        _create_two_player_game(ws, "game-spectator")
        ws.send_json(
            {
                "action": "subscribe_game",
                "payload": {"game_id": "game-spectator"},
            }
        )
        assert ws.receive_json()["type"] == "state_snapshot"

        ws.send_json(
            {
                "action": "request_legal_moves",
                "payload": {
                    "game_id": "game-spectator",
                    "player_id": "player-one",
                    "color": "blue",
                },
            }
        )
        assert ws.receive_json()["type"] == "legal_moves"

        ws.send_json(
            {
                "action": "place_move",
                "payload": {
                    "game_id": "game-spectator",
                    "command_id": "cmd-1",
                    "player_id": "player-one",
                    "color": "blue",
                    "piece_id": 0,
                    "orientation_id": 0,
                    "row": 0,
                    "col": 0,
                },
            }
        )
        event = ws.receive_json()

    assert event["type"] == "error"
    assert event["code"] == "not_seated"


def test_seat_takeover_evicts_existing_connection() -> None:
    client = make_client()

    with client.websocket_connect("/ws") as first:
        _create_two_player_game(first, "game-takeover")
        first.send_json(
            {
                "action": "subscribe_game",
                "payload": {"game_id": "game-takeover", "player_id": "player-one"},
            }
        )
        assert first.receive_json()["type"] == "state_snapshot"

        with client.websocket_connect("/ws") as second:
            second.send_json(
                {
                    "action": "subscribe_game",
                    "payload": {"game_id": "game-takeover", "player_id": "player-one"},
                }
            )
            kicked = first.receive_json()
            second_snapshot = second.receive_json()

            assert kicked["type"] == "kicked"
            assert kicked["reason"] == "seat_taken_by_reconnect"
            assert second_snapshot["type"] == "state_snapshot"

            second.send_json(
                {
                    "action": "place_move",
                    "payload": {
                        "game_id": "game-takeover",
                        "command_id": "cmd-1",
                        "player_id": "player-one",
                        "color": "blue",
                        "piece_id": 0,
                        "orientation_id": 0,
                        "row": 0,
                        "col": 0,
                    },
                }
            )
            event = second.receive_json()

    assert event["type"] == "move_applied"
