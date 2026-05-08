from __future__ import annotations

import logging
from collections import defaultdict
from typing import Any

from fastapi import WebSocket, WebSocketDisconnect

from blocus_backend.event_bus import EventSubscription, GameEventBus
from blocus_backend.service import GameService, ProtocolError

log = logging.getLogger(__name__)


class ConnectionManager:
    def __init__(self, event_bus: GameEventBus) -> None:
        self._event_bus = event_bus
        self._subscriptions: dict[str, set[WebSocket]] = defaultdict(set)
        self._event_subscriptions: dict[str, EventSubscription] = {}

    async def subscribe(self, game_id: str, websocket: WebSocket) -> None:
        if game_id not in self._event_subscriptions:
            self._event_subscriptions[game_id] = await self._event_bus.subscribe(
                game_id,
                lambda event: self._send_local(game_id, event),
            )
        self._subscriptions[game_id].add(websocket)

    async def disconnect(self, websocket: WebSocket) -> None:
        empty_games: list[str] = []
        for game_id, sockets in self._subscriptions.items():
            sockets.discard(websocket)
            if not sockets:
                empty_games.append(game_id)

        for game_id in empty_games:
            del self._subscriptions[game_id]
            event_subscription = self._event_subscriptions.pop(game_id, None)
            if event_subscription is not None:
                await event_subscription.close()

    async def broadcast(self, game_id: str, event: dict[str, Any]) -> None:
        await self._event_bus.publish(game_id, event)

    async def _send_local(self, game_id: str, event: dict[str, Any]) -> None:
        dead: list[WebSocket] = []
        for websocket in list(self._subscriptions.get(game_id, set())):
            try:
                await websocket.send_json(event)
            except RuntimeError:
                dead.append(websocket)
        for websocket in dead:
            await self.disconnect(websocket)


async def websocket_endpoint(
    websocket: WebSocket,
    service: GameService,
    manager: ConnectionManager,
) -> None:
    await websocket.accept()
    try:
        while True:
            message = await websocket.receive_json()
            await _handle_message(websocket, service, manager, message)
    except WebSocketDisconnect:
        await manager.disconnect(websocket)


async def _handle_message(
    websocket: WebSocket,
    service: GameService,
    manager: ConnectionManager,
    message: Any,
) -> None:
    if not isinstance(message, dict):
        await websocket.send_json(_error("invalid_message", "message must be a JSON object"))
        return

    action = message.get("action")
    payload = message.get("payload", {})
    if not isinstance(payload, dict):
        await websocket.send_json(_error("invalid_message", "payload must be an object"))
        return

    try:
        if action == "create_game":
            event = await service.create_game(payload)
            await manager.subscribe(event["game_id"], websocket)
            await manager.broadcast(event["game_id"], event)
        elif action in {"subscribe_game", "join_game"}:
            game_id = _game_id(payload)
            await manager.subscribe(game_id, websocket)
            snapshot = await service.state_snapshot(game_id)
            await websocket.send_json(snapshot)
        elif action == "request_state":
            await websocket.send_json(await service.state_snapshot(_game_id(payload)))
        elif action == "request_legal_moves":
            await websocket.send_json(await service.legal_moves(payload))
        elif action == "place_move":
            event = await service.place_move(payload)
            await _broadcast_with_ai_follow_up(service, manager, event)
        elif action == "pass_move":
            event = await service.pass_move(payload)
            await _broadcast_with_ai_follow_up(service, manager, event)
        elif action == "request_score":
            await websocket.send_json(await service.score(_game_id(payload)))
        elif action == "attach_ai":
            event = await service.attach_ai(payload)
            await _broadcast_with_ai_follow_up(service, manager, event)
        else:
            await websocket.send_json(_error("unknown_action", f"Unknown action: {action}"))
    except ProtocolError as error:
        await websocket.send_json(_error(error.code, error.message))
    except Exception:
        log.exception("Unhandled error processing websocket action %r", action)
        await websocket.send_json(_error("internal_error", "internal error"))


def _game_id(payload: dict[str, Any]) -> str:
    game_id = payload.get("game_id")
    if not isinstance(game_id, str) or not game_id:
        raise ProtocolError("missing_field", "Missing required field: game_id")
    return game_id


async def _broadcast_with_ai_follow_up(
    service: GameService,
    manager: ConnectionManager,
    event: dict[str, Any],
) -> None:
    await manager.broadcast(event["game_id"], event)
    for ai_event in await service.advance_ai_turns(event["game_id"]):
        await manager.broadcast(ai_event["game_id"], ai_event)


def _error(code: str, message: str) -> dict[str, str]:
    return {"type": "error", "code": code, "message": message}
