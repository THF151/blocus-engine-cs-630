from __future__ import annotations

import json
import logging
import os
from dataclasses import dataclass
from typing import Any, Protocol

log = logging.getLogger(__name__)


@dataclass(frozen=True)
class GameRecord:
    game_id: str
    state_json: str
    metadata: dict[str, Any]
    version: int


@dataclass(frozen=True)
class GameSummary:
    game_id: str
    name: str | None
    mode: str
    scoring: str
    players: dict[str, Any]
    created_at: str | None
    version: int


def _summary_from(game_id: str, metadata: dict[str, Any], version: int) -> GameSummary:
    return GameSummary(
        game_id=game_id,
        name=metadata.get("name"),
        mode=str(metadata.get("mode", "")),
        scoring=str(metadata.get("scoring", "")),
        players=metadata.get("players", {}),
        created_at=metadata.get("created_at"),
        version=version,
    )


class GameNotFoundError(KeyError):
    pass


class OptimisticLockError(RuntimeError):
    """Raised when save_game is called with a stale expected_version.

    The version is a repository-internal write counter (bumped on every save,
    state or metadata). It is *not* the engine's state.version — metadata-only
    writes change record.version too, which is what protects attach_ai from
    last-writer-wins.
    """

    def __init__(self, game_id: str, expected_version: int | None) -> None:
        super().__init__(
            f"Concurrent update on game {game_id!r}: expected version {expected_version}"
        )
        self.game_id = game_id
        self.expected_version = expected_version


class GameRepository(Protocol):
    async def save_game(
        self,
        game_id: str,
        state_json: str,
        metadata: dict[str, Any],
        *,
        expected_version: int | None,
    ) -> None: ...

    async def get_game(self, game_id: str) -> GameRecord: ...

    async def list_game_summaries(self) -> list[GameSummary]: ...


class InMemoryGameRepository:
    def __init__(self) -> None:
        self._records: dict[str, GameRecord] = {}

    async def save_game(
        self,
        game_id: str,
        state_json: str,
        metadata: dict[str, Any],
        *,
        expected_version: int | None,
    ) -> None:
        existing = self._records.get(game_id)
        current_version = existing.version if existing is not None else 0
        if expected_version is not None:
            if existing is None or existing.version != expected_version:
                raise OptimisticLockError(game_id, expected_version)
        new_version = current_version + 1
        self._records[game_id] = GameRecord(game_id, state_json, dict(metadata), new_version)

    async def get_game(self, game_id: str) -> GameRecord:
        try:
            return self._records[game_id]
        except KeyError as error:
            raise GameNotFoundError(game_id) from error

    async def list_game_summaries(self) -> list[GameSummary]:
        return [
            _summary_from(record.game_id, record.metadata, record.version)
            for record in self._records.values()
        ]


_CAS_SCRIPT = """
local current = redis.call('HGET', KEYS[1], 'version')
local expected = ARGV[1]
local new_version
if current == false then
  if expected ~= '' then
    return 0
  end
  new_version = 1
elseif expected == '' or tonumber(current) == tonumber(expected) then
  new_version = tonumber(current) + 1
else
  return 0
end
redis.call('HSET', KEYS[1], 'state', ARGV[2], 'metadata', ARGV[3], 'version', tostring(new_version))
return 1
"""


class RedisGameRepository:
    def __init__(self, url: str | None = None, redis_client: Any | None = None) -> None:
        if redis_client is None:
            redis_url = (
                url
                if url is not None
                else os.getenv("BLOCUS_REDIS_URL", "redis://localhost:6379/0")
            )
            try:
                from redis.asyncio import Redis
            except ModuleNotFoundError as error:
                raise RuntimeError(
                    "Install the redis package to use RedisGameRepository"
                ) from error
            redis_client = Redis.from_url(redis_url, decode_responses=True)

        self._redis: Any = redis_client

    async def save_game(
        self,
        game_id: str,
        state_json: str,
        metadata: dict[str, Any],
        *,
        expected_version: int | None,
    ) -> None:
        key = _game_key(game_id)
        expected_arg = "" if expected_version is None else str(expected_version)
        result = await self._redis.eval(
            _CAS_SCRIPT,
            1,
            key,
            expected_arg,
            state_json,
            json.dumps(metadata),
        )
        if int(result) != 1:
            raise OptimisticLockError(game_id, expected_version)

        # Index the game id so list_game_summaries can enumerate without KEYS.
        # SADD is idempotent; a stale id with no hash is skipped on read.
        await self._redis.sadd(_INDEX_KEY, game_id)

    async def get_game(self, game_id: str) -> GameRecord:
        data = await self._redis.hgetall(_game_key(game_id))
        if not data:
            raise GameNotFoundError(game_id)

        return GameRecord(
            game_id=game_id,
            state_json=data["state"],
            metadata=json.loads(data["metadata"]),
            version=int(data["version"]),
        )

    async def list_game_summaries(self) -> list[GameSummary]:
        game_ids = await self._redis.smembers(_INDEX_KEY)
        summaries: list[GameSummary] = []
        for game_id in game_ids:
            metadata_raw, version_raw = await self._redis.hmget(
                _game_key(game_id), "metadata", "version"
            )
            if metadata_raw is None or version_raw is None:
                continue  # stale index entry; the game hash is gone
            summaries.append(_summary_from(game_id, json.loads(metadata_raw), int(version_raw)))
        return summaries

    async def aclose(self) -> None:
        aclose = getattr(self._redis, "aclose", None)
        if aclose is not None:
            await aclose()


_INDEX_KEY = "blocus:games"


def _game_key(game_id: str) -> str:
    return f"blocus:game:{game_id}"
