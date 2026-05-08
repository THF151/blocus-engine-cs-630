from __future__ import annotations

import json
from typing import Any
from uuid import uuid4

from blocus_backend.engine_adapter import ApplyResult
from blocus_backend.repository import GameNotFoundError, GameRecord, GameRepository

CLASSIC_COLORS = ["blue", "yellow", "red", "green"]
CLASSIC_MODES = {"two_player", "three_player", "four_player"}
MAX_AI_TURNS = 10_000


class ProtocolError(ValueError):
    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code
        self.message = message


class GameService:
    def __init__(self, repository: GameRepository, engine: Any) -> None:
        self._repository = repository
        self._engine = engine

    async def create_game(self, payload: dict[str, Any]) -> dict[str, Any]:
        normalized = _normalize_create_game(payload)
        state = self._engine.create_game(normalized)
        metadata = _metadata_from_config(normalized)
        await self._repository.save_game(
            normalized["game_id"],
            self._engine.serialize_state(state),
            metadata,
        )
        return _event("game_created", normalized["game_id"], self._engine.state_view(state))

    async def state_snapshot(self, game_id: str) -> dict[str, Any]:
        record = await self._record(game_id)
        state = self._engine.deserialize_state(record.state_json)
        return _event("state_snapshot", game_id, self._engine.state_view(state))

    async def legal_moves(self, payload: dict[str, Any]) -> dict[str, Any]:
        game_id = _required_str(payload, "game_id")
        player_id = _required_str(payload, "player_id")
        color = _classic_color(_required_str(payload, "color"))
        record = await self._record(game_id)
        state = self._engine.deserialize_state(record.state_json)
        return {
            "type": "legal_moves",
            "game_id": game_id,
            "player_id": player_id,
            "color": color,
            "moves": self._engine.legal_moves(state, player_id, color),
        }

    async def place_move(self, payload: dict[str, Any]) -> dict[str, Any]:
        game_id = _required_str(payload, "game_id")
        payload["color"] = _classic_color(_required_str(payload, "color"))
        record = await self._record(game_id)
        state = self._engine.deserialize_state(record.state_json)
        result = self._engine.place_move(state, payload)
        return await self._persist_apply_result(game_id, record, result)

    async def pass_move(self, payload: dict[str, Any]) -> dict[str, Any]:
        game_id = _required_str(payload, "game_id")
        payload["color"] = _classic_color(_required_str(payload, "color"))
        record = await self._record(game_id)
        state = self._engine.deserialize_state(record.state_json)
        result = self._engine.pass_move(state, payload)
        return await self._persist_apply_result(game_id, record, result)

    async def score(self, game_id: str) -> dict[str, Any]:
        record = await self._record(game_id)
        state = self._engine.deserialize_state(record.state_json)
        return {
            "type": "score_report",
            "game_id": game_id,
            "score": self._engine.score_game(state),
        }

    async def attach_ai(self, payload: dict[str, Any]) -> dict[str, Any]:
        game_id = _required_str(payload, "game_id")
        player_id = _required_str(payload, "player_id")
        color = _classic_color(_required_str(payload, "color"))

        record = await self._record(game_id)
        metadata = dict(record.metadata)
        ai_seats = list(metadata.get("ai_seats", []))
        seat = {"player_id": player_id, "color": color}
        if seat not in ai_seats:
            ai_seats.append(seat)
        metadata["ai_seats"] = ai_seats
        await self._repository.update_metadata(game_id, metadata)

        state = self._engine.deserialize_state(record.state_json)
        return _event("game_joined", game_id, self._engine.state_view(state))

    async def advance_ai_turns(self, game_id: str) -> list[dict[str, Any]]:
        events: list[dict[str, Any]] = []

        for _ in range(MAX_AI_TURNS):
            record = await self._record(game_id)
            state = self._engine.deserialize_state(record.state_json)
            view = self._engine.state_view(state)
            if view["status"] == "finished":
                return events

            color = str(view["current_color"])
            seat = _ai_seat_for_color(record.metadata, color, record.state_json)
            if seat is None:
                return events

            moves = self._engine.legal_moves(state, seat["player_id"], color)
            command_payload = {
                "game_id": game_id,
                "command_id": str(uuid4()),
                "player_id": seat["player_id"],
                "color": color,
            }
            if moves:
                command_payload.update(moves[0])
                result = self._engine.place_move(state, command_payload)
            else:
                result = self._engine.pass_move(state, command_payload)

            event = await self._persist_apply_result(game_id, record, result)
            events.append(event)
            if event["type"] == "game_finished":
                return events

        raise ProtocolError("ai_turn_limit_exceeded", "AI turn limit exceeded")

    async def _record(self, game_id: str) -> GameRecord:
        try:
            return await self._repository.get_game(game_id)
        except GameNotFoundError as error:
            raise ProtocolError("game_not_found", f"Game {game_id!r} was not found") from error

    async def _persist_apply_result(
        self,
        game_id: str,
        record: GameRecord,
        result: ApplyResult,
    ) -> dict[str, Any]:
        state_json = self._engine.serialize_state(result.next_state)
        await self._repository.save_game(game_id, state_json, record.metadata)
        state_view = self._engine.state_view(result.next_state)
        event = _event(result.event_type, game_id, state_view)
        event["response"] = result.response
        if state_view["status"] == "finished":
            event["type"] = "game_finished"
        return event


def _ai_seat_for_color(
    metadata: dict[str, Any],
    color: str,
    state_json: str | None = None,
) -> dict[str, str] | None:
    raw_seats = metadata.get("ai_seats", [])
    if not isinstance(raw_seats, list):
        return None

    if metadata.get("mode") == "three_player" and color == "green":
        scheduled_player = _scheduled_shared_green_player(metadata, state_json)
        if scheduled_player is None:
            return None
        return _matching_ai_seat(raw_seats, scheduled_player, color)

    return _first_ai_seat_for_color(raw_seats, color)


def _first_ai_seat_for_color(raw_seats: list[Any], color: str) -> dict[str, str] | None:
    for raw_seat in raw_seats:
        if not isinstance(raw_seat, dict):
            continue
        player_id = raw_seat.get("player_id")
        seat_color = raw_seat.get("color")
        if isinstance(player_id, str) and seat_color == color:
            return {"player_id": player_id, "color": color}

    return None


def _matching_ai_seat(
    raw_seats: list[Any],
    player_id: str,
    color: str,
) -> dict[str, str] | None:
    for raw_seat in raw_seats:
        if not isinstance(raw_seat, dict):
            continue
        seat_player_id = raw_seat.get("player_id")
        seat_color = raw_seat.get("color")
        if seat_player_id == player_id and seat_color == color:
            return {"player_id": player_id, "color": color}

    return None


def _scheduled_shared_green_player(
    metadata: dict[str, Any],
    state_json: str | None,
) -> str | None:
    if state_json is None:
        return None

    players = metadata.get("players")
    if not isinstance(players, dict):
        return None
    shared_green = players.get("shared_green")
    if not isinstance(shared_green, list) or not shared_green:
        return None

    try:
        data = json.loads(state_json)
    except TypeError, ValueError:
        return None
    if not isinstance(data, dict):
        return None

    turn = data.get("turn")
    if not isinstance(turn, dict):
        return None
    raw_index = turn.get("shared_color_turn_index", 0)
    if not isinstance(raw_index, int):
        return None

    player_id = shared_green[raw_index % len(shared_green)]
    if not isinstance(player_id, str):
        return None
    return player_id


def _normalize_create_game(payload: dict[str, Any]) -> dict[str, Any]:
    mode = _required_str(payload, "mode")
    if mode not in CLASSIC_MODES:
        raise ProtocolError("invalid_classic_mode", "Only Classic modes are supported")

    game_id = str(payload.get("game_id") or uuid4())
    scoring = str(payload.get("scoring", "basic"))
    if scoring not in {"basic", "advanced"}:
        raise ProtocolError("invalid_scoring", "Scoring must be basic or advanced")

    if mode == "two_player":
        players = dict(payload.get("players", {}))
        _require_keys(players, {"blue_red", "yellow_green"})
        turn_order = list(CLASSIC_COLORS)
    elif mode == "three_player":
        players = dict(payload.get("players", {}))
        _require_keys(players, {"blue", "yellow", "red", "shared_green"})
        if not isinstance(players["shared_green"], list) or not players["shared_green"]:
            raise ProtocolError("invalid_players", "shared_green must be a non-empty player list")
        turn_order = list(CLASSIC_COLORS)
    else:
        players = dict(payload.get("players", {}))
        _require_keys(players, set(CLASSIC_COLORS))
        first_color = _classic_color(str(payload.get("first_color", "blue")))
        turn_order = _rotated_turn_order(first_color)

    return {
        "game_id": game_id,
        "mode": mode,
        "scoring": scoring,
        "players": players,
        "turn_order": turn_order,
    }


def _metadata_from_config(config: dict[str, Any]) -> dict[str, Any]:
    return {
        "mode": config["mode"],
        "scoring": config["scoring"],
        "players": config["players"],
        "turn_order": config["turn_order"],
        "ai_seats": [],
    }


def _event(event_type: str, game_id: str, state: dict[str, Any]) -> dict[str, Any]:
    return {"type": event_type, "game_id": game_id, "state": state}


def _rotated_turn_order(first_color: str) -> list[str]:
    start = CLASSIC_COLORS.index(first_color)
    return CLASSIC_COLORS[start:] + CLASSIC_COLORS[:start]


def _classic_color(value: str) -> str:
    if value not in CLASSIC_COLORS:
        raise ProtocolError("invalid_classic_color", "Only Classic colors are supported")
    return value


def _required_str(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise ProtocolError("missing_field", f"Missing required field: {key}")
    return value


def _require_keys(payload: dict[str, Any], keys: set[str]) -> None:
    missing = keys.difference(payload)
    if missing:
        raise ProtocolError("invalid_players", f"Missing player assignments: {sorted(missing)}")

    invalid_colors = set(payload).intersection({"black", "white"})
    if invalid_colors:
        raise ProtocolError("invalid_classic_color", "Duo colors are not valid in Classic mode")
