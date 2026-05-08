from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any

from blocus_backend.engine_adapter import ApplyResult


@dataclass
class FakeState:
    game_id: str
    mode: str
    current_color: str
    turn_order: list[str]
    version: int = 0
    status: str = "in_progress"


class FakeClassicEngine:
    def serialize_state(self, state: FakeState) -> str:
        return json.dumps(state.__dict__)

    def deserialize_state(self, text: str) -> FakeState:
        return FakeState(**json.loads(text))

    def create_game(self, payload: dict[str, Any]) -> FakeState:
        return FakeState(
            game_id=payload["game_id"],
            mode=payload["mode"],
            current_color=payload["turn_order"][0],
            turn_order=list(payload["turn_order"]),
        )

    def state_view(self, state: FakeState) -> dict[str, Any]:
        return {
            "game_id": state.game_id,
            "mode": state.mode,
            "status": state.status,
            "version": state.version,
            "current_color": state.current_color,
            "turn_order": state.turn_order,
            "board_size": 20,
        }

    def legal_moves(self, _state: FakeState, _player_id: str, _color: str) -> list[dict[str, int]]:
        return [
            {
                "piece_id": 0,
                "orientation_id": 0,
                "row": 0,
                "col": 0,
                "board_index": 0,
                "score_delta": 1,
            }
        ]

    def place_move(self, state: FakeState, _payload: dict[str, Any]) -> ApplyResult:
        next_state = FakeState(
            game_id=state.game_id,
            mode=state.mode,
            current_color=state.turn_order[(state.version + 1) % len(state.turn_order)],
            turn_order=state.turn_order,
            version=state.version + 1,
        )
        return ApplyResult(
            next_state=next_state, event_type="move_applied", response="move applied"
        )

    def pass_move(self, state: FakeState, _payload: dict[str, Any]) -> ApplyResult:
        next_state = FakeState(
            game_id=state.game_id,
            mode=state.mode,
            current_color=state.turn_order[(state.version + 1) % len(state.turn_order)],
            turn_order=state.turn_order,
            version=state.version + 1,
        )
        return ApplyResult(
            next_state=next_state, event_type="pass_applied", response="player passed"
        )

    def score_game(self, _state: FakeState) -> dict[str, Any]:
        return {"scoring": "basic", "entries": []}
