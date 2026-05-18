from __future__ import annotations

import os
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

from fastapi import FastAPI, WebSocket
from fastapi.middleware.cors import CORSMiddleware

from blocus_backend.engine_adapter import ClassicEngineAdapter
from blocus_backend.event_bus import GameEventBus, InMemoryGameEventBus, RedisGameEventBus
from blocus_backend.repository import GameRepository, InMemoryGameRepository, RedisGameRepository
from blocus_backend.service import GameService
from blocus_backend.websocket import ConnectionManager, websocket_endpoint

# Localhost-default keeps `make dev` and the Flutter web target working
# without env-var setup. Production deployments set BLOCUS_CORS_ORIGINS to
# an explicit comma-separated list; the regex default is *not* `*`, so a
# forgotten env var fails closed against external origins instead of
# silently allowing everything.
_DEV_ORIGIN_REGEX = r"^http://(localhost|127\.0\.0\.1)(:\d+)?$"


def create_app(
    repository: GameRepository | None = None,
    engine: Any | None = None,
    event_bus: GameEventBus | None = None,
) -> FastAPI:
    engine_adapter = engine or ClassicEngineAdapter()
    # Use Redis only when BLOCUS_REDIS_URL is explicitly set; otherwise fall
    # back to in-memory implementations so `make dev` works without Redis.
    _use_redis = bool(os.getenv("BLOCUS_REDIS_URL"))
    game_repository: GameRepository = repository or (
        RedisGameRepository() if _use_redis else InMemoryGameRepository()
    )
    game_event_bus: GameEventBus = event_bus or (
        RedisGameEventBus() if _use_redis else InMemoryGameEventBus()
    )
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

    cors_kwargs = _cors_kwargs(os.getenv("BLOCUS_CORS_ORIGINS"))
    app.add_middleware(
        CORSMiddleware,
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
        **cors_kwargs,
    )

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


def _cors_kwargs(env_value: str | None) -> dict[str, Any]:
    if env_value is None or not env_value.strip():
        return {"allow_origin_regex": _DEV_ORIGIN_REGEX}
    origins = [origin.strip() for origin in env_value.split(",") if origin.strip()]
    return {"allow_origins": origins}
