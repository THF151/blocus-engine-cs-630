from __future__ import annotations

from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

from fastapi import FastAPI, WebSocket

from blocus_backend.engine_adapter import ClassicEngineAdapter
from blocus_backend.event_bus import GameEventBus, RedisGameEventBus
from blocus_backend.repository import GameRepository, RedisGameRepository
from blocus_backend.service import GameService
from blocus_backend.websocket import ConnectionManager, websocket_endpoint


def create_app(
    repository: GameRepository | None = None,
    engine: Any | None = None,
    event_bus: GameEventBus | None = None,
) -> FastAPI:
    engine_adapter = engine or ClassicEngineAdapter()
    game_repository: GameRepository = repository or RedisGameRepository()
    game_event_bus: GameEventBus = event_bus or RedisGameEventBus()
    manager = ConnectionManager(game_event_bus)
    service = GameService(
        game_repository,
        engine_adapter,
        seat_binding_check=manager.is_seat_bound,
    )

    @asynccontextmanager
    async def lifespan(_: FastAPI) -> AsyncIterator[None]:
        try:
            yield
        finally:
            for resource in (game_event_bus, game_repository):
                aclose = getattr(resource, "aclose", None)
                if aclose is not None:
                    await aclose()

    app = FastAPI(title="Blocus Backend", lifespan=lifespan)

    @app.get("/health")
    def health() -> dict[str, bool | str]:
        return {
            "status": "ok",
            "engine": engine_adapter.engine_health(),
        }

    @app.websocket("/ws")
    async def websocket_route(websocket: WebSocket) -> None:
        await websocket_endpoint(websocket, service, manager)

    return app
