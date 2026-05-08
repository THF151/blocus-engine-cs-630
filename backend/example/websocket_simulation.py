from __future__ import annotations

import argparse
import json
from collections.abc import Iterator
from typing import Any
from uuid import uuid4

from websockets.sync.client import connect

DEFAULT_URL = "ws://localhost:8000/ws"


def main() -> None:
    parser = argparse.ArgumentParser(description="Run a Classic Blocus WebSocket simulation.")
    parser.add_argument("--url", default=DEFAULT_URL)
    parser.add_argument(
        "--mode", choices=["two_player", "three_player", "four_player"], default="two_player"
    )
    parser.add_argument("--ai-count", type=int, choices=[1, 2, 3], default=1)
    args = parser.parse_args()

    game_id = str(uuid4())
    players = _players_for_mode(args.mode)
    ai_seats = list(_ai_seats_for_mode(args.mode, args.ai_count))

    with connect(args.url) as websocket:
        _send(
            websocket,
            {
                "action": "create_game",
                "payload": {
                    "game_id": game_id,
                    "mode": args.mode,
                    "scoring": "basic",
                    "first_color": "blue",
                    "players": players,
                },
            },
        )
        event = _receive_until(websocket, {"game_created", "error"})
        if event["type"] == "error":
            raise RuntimeError(event)

        for player_id, color in ai_seats:
            _send(
                websocket,
                {
                    "action": "attach_ai",
                    "payload": {"game_id": game_id, "player_id": player_id, "color": color},
                },
            )

        current_event = _request_state(websocket, game_id)
        while current_event["state"]["status"] != "finished":
            state = current_event["state"]
            color = state["current_color"]
            player_id = _player_for_color(args.mode, players, color, state)

            _send(
                websocket,
                {
                    "action": "request_legal_moves",
                    "payload": {"game_id": game_id, "player_id": player_id, "color": color},
                },
            )
            moves_event = _receive_until(websocket, {"legal_moves", "error"})
            if moves_event["type"] == "error":
                raise RuntimeError(moves_event)

            if moves_event["moves"]:
                move = moves_event["moves"][0]
                _send(
                    websocket,
                    {
                        "action": "place_move",
                        "payload": {
                            "game_id": game_id,
                            "command_id": str(uuid4()),
                            "player_id": player_id,
                            "color": color,
                            **move,
                        },
                    },
                )
            else:
                _send(
                    websocket,
                    {
                        "action": "pass_move",
                        "payload": {
                            "game_id": game_id,
                            "command_id": str(uuid4()),
                            "player_id": player_id,
                            "color": color,
                        },
                    },
                )

            action_event = _receive_until(
                websocket,
                {"move_applied", "pass_applied", "game_finished", "error"},
            )
            if action_event["type"] == "error":
                raise RuntimeError(action_event)

            print(
                f"{action_event['type']}: "
                f"version={action_event['state']['version']} "
                f"current={action_event['state']['current_color']}"
            )
            current_event = _request_state(websocket, game_id)

        _send(websocket, {"action": "request_score", "payload": {"game_id": game_id}})
        print(json.dumps(_receive_until(websocket, {"score_report", "error"}), indent=2))


def _send(websocket: Any, message: dict[str, Any]) -> None:
    websocket.send(json.dumps(message))


def _receive_until(websocket: Any, event_types: set[str]) -> dict[str, Any]:
    while True:
        event = json.loads(websocket.recv())
        if event["type"] in event_types:
            return event


def _request_state(websocket: Any, game_id: str) -> dict[str, Any]:
    _send(websocket, {"action": "request_state", "payload": {"game_id": game_id}})
    event = _receive_until(websocket, {"state_snapshot", "error"})
    if event["type"] == "error":
        raise RuntimeError(event)
    return event


def _players_for_mode(mode: str) -> dict[str, Any]:
    if mode == "two_player":
        return {"blue_red": _player_id(1), "yellow_green": _player_id(2)}
    if mode == "three_player":
        return {
            "blue": _player_id(1),
            "yellow": _player_id(2),
            "red": _player_id(3),
            "shared_green": [_player_id(1), _player_id(2), _player_id(3)],
        }
    return {
        "blue": _player_id(1),
        "yellow": _player_id(2),
        "red": _player_id(3),
        "green": _player_id(4),
    }


def _ai_seats_for_mode(mode: str, ai_count: int) -> Iterator[tuple[str, str]]:
    if mode == "two_player":
        seat_groups = [
            [(_player_id(1), "blue"), (_player_id(1), "red")],
            [(_player_id(2), "yellow"), (_player_id(2), "green")],
        ]
    elif mode == "three_player":
        seat_groups = [
            [(_player_id(1), "blue"), (_player_id(1), "green")],
            [(_player_id(2), "yellow"), (_player_id(2), "green")],
            [(_player_id(3), "red"), (_player_id(3), "green")],
        ]
    else:
        seat_groups = [
            [(_player_id(1), "blue")],
            [(_player_id(2), "yellow")],
            [(_player_id(3), "red")],
            [(_player_id(4), "green")],
        ]

    for group in seat_groups[:ai_count]:
        yield from group


def _player_for_color(mode: str, players: dict[str, Any], color: str, state: dict[str, Any]) -> str:
    if mode == "two_player":
        return players["blue_red"] if color in {"blue", "red"} else players["yellow_green"]
    if mode == "three_player":
        if color == "green":
            shared_green = players["shared_green"]
            shared_index = int(state.get("shared_color_turn_index", 0))
            return shared_green[shared_index % len(shared_green)]
        return players[color]
    return players[color]


def _player_id(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


if __name__ == "__main__":
    main()
