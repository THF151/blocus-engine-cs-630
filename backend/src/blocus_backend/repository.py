from __future__ import annotations

import json
import os
from dataclasses import dataclass
from typing import Any, Protocol


@dataclass(frozen=True)
class GameRecord:
    game_id: str
    state_json: str
    metadata: dict[str, Any]


class GameNotFoundError(KeyError):
    pass


class GameRepository(Protocol):
    async def save_game(self, game_id: str, state_json: str, metadata: dict[str, Any]) -> None: ...

    async def get_game(self, game_id: str) -> GameRecord: ...

    async def update_metadata(self, game_id: str, metadata: dict[str, Any]) -> None: ...


class InMemoryGameRepository:
    def __init__(self) -> None:
        self._records: dict[str, GameRecord] = {}

    async def save_game(self, game_id: str, state_json: str, metadata: dict[str, Any]) -> None:
        self._records[game_id] = GameRecord(game_id, state_json, dict(metadata))

    async def get_game(self, game_id: str) -> GameRecord:
        try:
            return self._records[game_id]
        except KeyError as error:
            raise GameNotFoundError(game_id) from error

    async def update_metadata(self, game_id: str, metadata: dict[str, Any]) -> None:
        record = await self.get_game(game_id)
        self._records[game_id] = GameRecord(game_id, record.state_json, dict(metadata))


class RedisGameRepository:
    def __init__(self, url: str | None = None) -> None:
        redis_url: str = (
            url
            if url is not None
            else os.getenv(
                "BLOCUS_REDIS_URL",
                "redis://localhost:6379/0",
            )
        )
        try:
            from redis.asyncio import Redis
        except ModuleNotFoundError as error:
            raise RuntimeError("Install the redis package to use RedisGameRepository") from error

        self._redis: Any = Redis.from_url(redis_url, decode_responses=True)

    async def save_game(self, game_id: str, state_json: str, metadata: dict[str, Any]) -> None:
        await self._redis.hset(
            _game_key(game_id),
            mapping={
                "state": state_json,
                "metadata": json.dumps(metadata),
            },
        )

    async def get_game(self, game_id: str) -> GameRecord:
        data = await self._redis.hgetall(_game_key(game_id))
        if not data:
            raise GameNotFoundError(game_id)

        return GameRecord(
            game_id=game_id,
            state_json=data["state"],
            metadata=json.loads(data["metadata"]),
        )

    async def update_metadata(self, game_id: str, metadata: dict[str, Any]) -> None:
        record = await self.get_game(game_id)
        await self.save_game(game_id, record.state_json, metadata)

    async def aclose(self) -> None:
        aclose = getattr(self._redis, "aclose", None)
        if aclose is not None:
            await aclose()


def _game_key(game_id: str) -> str:
    return f"blocus:game:{game_id}"
