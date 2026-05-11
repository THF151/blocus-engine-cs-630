from __future__ import annotations

import json
import logging
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from typing import Any
from uuid import uuid4

from pydantic import BaseModel, ValidationError

from blocus_backend.engine_adapter import ApplyResult
from blocus_backend.repository import (
    GameNotFoundError,
    GameRecord,
    GameRepository,
    OptimisticLockError,
)
from blocus_backend.schemas import (
    AttachAiRequest,
    FourPlayerCreate,
    LegalMovesRequest,
    PassMoveRequest,
    PlaceMoveRequest,
    ThreePlayerCreate,
    TwoPlayerCreate,
)

log = logging.getLogger(__name__)

CLASSIC_COLORS = ["blue", "yellow", "red", "green"]
CLASSIC_MODES = {"two_player", "three_player", "four_player"}
SCORING_MODES = {"basic", "advanced"}
MAX_AI_TURNS = 10_000


try:
    from blocus_engine import (
        EngineError as _EngineError,  # type: ignore[import-not-found,unused-ignore]
    )
    from blocus_engine import (
        InputError as _InputError,  # type: ignore[import-not-found,unused-ignore]
    )
    from blocus_engine import (
        RuleViolationError as _RuleViolationError,  # type: ignore[import-not-found,unused-ignore]
    )
except ModuleNotFoundError:

    class _InputError(Exception):  # type: ignore[no-redef]
        pass

    class _RuleViolationError(Exception):  # type: ignore[no-redef]
        pass

    class _EngineError(Exception):  # type: ignore[no-redef]
        pass


class ProtocolError(ValueError):
    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code
        self.message = message


@contextmanager
def _map_engine_errors() -> Iterator[None]:
    try:
        yield
    except _RuleViolationError as error:
        raise ProtocolError("rule_violation", str(error)) from error
    except _InputError as error:
        raise ProtocolError("invalid_command", str(error)) from error
    except _EngineError as error:
        log.exception("engine error")
        raise ProtocolError("internal_error", "engine error") from error


class GameService:
    def __init__(
        self,
        repository: GameRepository,
        engine: Any,
        seat_binding_check: Callable[[str, str], bool] | None = None,
    ) -> None:
        self._repository = repository
        self._engine = engine
        self._is_seat_bound = seat_binding_check or (lambda _g, _p: False)

    async def create_game(self, payload: dict[str, Any]) -> dict[str, Any]:
        mode = payload.get("mode")
        if mode == "two_player":
            normalized = _normalize_two_player(_parse(payload, TwoPlayerCreate))
        elif mode == "three_player":
            normalized = _normalize_three_player(_parse(payload, ThreePlayerCreate))
        elif mode == "four_player":
            normalized = _normalize_four_player(_parse(payload, FourPlayerCreate))
        else:
            raise ProtocolError("invalid_classic_mode", "Only Classic modes are supported")

        with _map_engine_errors():
            state = self._engine.create_game(normalized)
            state_json = self._engine.serialize_state(state)
        metadata = _metadata_from_config(normalized)
        await self._repository.save_game(
            normalized["game_id"],
            state_json,
            metadata,
            expected_version=None,
        )
        with _map_engine_errors():
            view = self._engine.state_view(state)
        return _event("game_created", normalized["game_id"], view)

    async def state_snapshot(self, game_id: str) -> dict[str, Any]:
        record = await self._record(game_id)
        with _map_engine_errors():
            state = self._engine.deserialize_state(record.state_json)
            view = self._engine.state_view(state)
        return _event("state_snapshot", game_id, view)

    async def legal_moves(self, payload: dict[str, Any]) -> dict[str, Any]:
        request = _parse(payload, LegalMovesRequest)
        color = _classic_color(request.color)
        record = await self._record(request.game_id)
        with _map_engine_errors():
            state = self._engine.deserialize_state(record.state_json)
            moves = self._engine.legal_moves(state, request.player_id, color)
        return {
            "type": "legal_moves",
            "game_id": request.game_id,
            "player_id": request.player_id,
            "color": color,
            "moves": moves,
        }

    async def place_move(self, payload: dict[str, Any]) -> dict[str, Any]:
        request = _parse(payload, PlaceMoveRequest)
        color = _classic_color(request.color)
        record = await self._record(request.game_id)
        with _map_engine_errors():
            state = self._engine.deserialize_state(record.state_json)
            command_payload = {**request.model_dump(), "color": color}
            result = self._engine.place_move(state, command_payload)
        try:
            return await self._persist_apply_result(request.game_id, record, result)
        except OptimisticLockError as error:
            raise ProtocolError("conflict", "Game state changed concurrently — retry") from error

    async def pass_move(self, payload: dict[str, Any]) -> dict[str, Any]:
        request = _parse(payload, PassMoveRequest)
        color = _classic_color(request.color)
        record = await self._record(request.game_id)
        with _map_engine_errors():
            state = self._engine.deserialize_state(record.state_json)
            command_payload = {**request.model_dump(), "color": color}
            result = self._engine.pass_move(state, command_payload)
        try:
            return await self._persist_apply_result(request.game_id, record, result)
        except OptimisticLockError as error:
            raise ProtocolError("conflict", "Game state changed concurrently — retry") from error

    async def score(self, game_id: str) -> dict[str, Any]:
        record = await self._record(game_id)
        with _map_engine_errors():
            state = self._engine.deserialize_state(record.state_json)
            score = self._engine.score_game(state)
        return {
            "type": "score_report",
            "game_id": game_id,
            "score": score,
        }

    async def attach_ai(self, payload: dict[str, Any]) -> dict[str, Any]:
        request = _parse(payload, AttachAiRequest)
        color = _classic_color(request.color)

        record = await self._record(request.game_id)
        metadata = dict(record.metadata)
        ai_seats = list(metadata.get("ai_seats", []))
        seat = {"player_id": request.player_id, "color": color}
        if seat not in ai_seats:
            ai_seats.append(seat)
        metadata["ai_seats"] = ai_seats
        try:
            await self._repository.save_game(
                request.game_id,
                record.state_json,
                metadata,
                expected_version=record.version,
            )
        except OptimisticLockError as error:
            raise ProtocolError("conflict", "Game state changed concurrently — retry") from error

        with _map_engine_errors():
            state = self._engine.deserialize_state(record.state_json)
            view = self._engine.state_view(state)
        return _event("game_joined", request.game_id, view)

    async def advance_ai_turns(self, game_id: str) -> list[dict[str, Any]]:
        events: list[dict[str, Any]] = []

        for _ in range(MAX_AI_TURNS):
            record = await self._record(game_id)
            with _map_engine_errors():
                state = self._engine.deserialize_state(record.state_json)
                view = self._engine.state_view(state)
            if view["status"] == "finished":
                return events

            color = str(view["current_color"])
            seat = _ai_seat_for_color(record.metadata, color, record.state_json)
            if seat is None:
                return events
            # Binding ≻ AI: if a human is bound to this seat on this worker,
            # the AI yields control. Cross-worker takeover relies on sticky
            # sessions — see PROTOCOL.md.
            if self._is_seat_bound(game_id, seat["player_id"]):
                return events

            command_payload = {
                "game_id": game_id,
                "command_id": str(uuid4()),
                "player_id": seat["player_id"],
                "color": color,
            }
            with _map_engine_errors():
                moves = self._engine.legal_moves(state, seat["player_id"], color)
                if moves:
                    command_payload.update(moves[0])
                    result = self._engine.place_move(state, command_payload)
                else:
                    result = self._engine.pass_move(state, command_payload)

            try:
                event = await self._persist_apply_result(game_id, record, result)
            except OptimisticLockError:
                # Another writer raced us; loop re-reads the latest state and
                # the AI re-derives its move from there.
                continue
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
        with _map_engine_errors():
            state_json = self._engine.serialize_state(result.next_state)
        await self._repository.save_game(
            game_id,
            state_json,
            record.metadata,
            expected_version=record.version,
        )
        with _map_engine_errors():
            state_view = self._engine.state_view(result.next_state)
        event = _event(result.event_type, game_id, state_view)
        event["response"] = result.response
        if state_view["status"] == "finished":
            event["type"] = "game_finished"
        return event


def _parse[ModelT: BaseModel](payload: dict[str, Any], model: type[ModelT]) -> ModelT:
    try:
        return model.model_validate(payload)
    except ValidationError as error:
        raise _validation_error_to_protocol_error(error) from error


def _validation_error_to_protocol_error(error: ValidationError) -> ProtocolError:
    first = error.errors()[0]
    loc: tuple[Any, ...] = first.get("loc", ())
    msg = first.get("msg", "validation failed")

    if loc and loc[0] == "players":
        return ProtocolError("invalid_players", f"Invalid players: {msg}")

    field = ".".join(str(x) for x in loc) if loc else "(unknown)"
    return ProtocolError("missing_field", f"Missing or invalid field: {field}")


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


def _normalize_two_player(request: TwoPlayerCreate) -> dict[str, Any]:
    return {
        "game_id": request.game_id or str(uuid4()),
        "mode": "two_player",
        "scoring": _scoring(request.scoring),
        "players": request.players.model_dump(),
        "turn_order": list(CLASSIC_COLORS),
    }


def _normalize_three_player(request: ThreePlayerCreate) -> dict[str, Any]:
    return {
        "game_id": request.game_id or str(uuid4()),
        "mode": "three_player",
        "scoring": _scoring(request.scoring),
        "players": request.players.model_dump(),
        "turn_order": list(CLASSIC_COLORS),
    }


def _normalize_four_player(request: FourPlayerCreate) -> dict[str, Any]:
    first_color = _classic_color(request.first_color)
    return {
        "game_id": request.game_id or str(uuid4()),
        "mode": "four_player",
        "scoring": _scoring(request.scoring),
        "players": request.players.model_dump(),
        "turn_order": _rotated_turn_order(first_color),
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


def _scoring(value: str) -> str:
    if value not in SCORING_MODES:
        raise ProtocolError("invalid_scoring", "Scoring must be basic or advanced")
    return value
