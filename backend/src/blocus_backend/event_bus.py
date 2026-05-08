from __future__ import annotations

import asyncio
import json
import os
from collections import defaultdict
from collections.abc import Awaitable, Callable
from typing import Any, Protocol

GAME_EVENT_CHANNEL_PREFIX = "blocus:game_events:"
GAME_EVENT_STREAM_PREFIX = "blocus:game_stream:"
STREAM_MAXLEN = 1_000

EventCallback = Callable[[dict[str, Any]], Awaitable[None]]


class EventSubscription(Protocol):
    async def close(self) -> None: ...


class GameEventBus(Protocol):
    async def publish(self, game_id: str, event: dict[str, Any]) -> None: ...

    async def subscribe(self, game_id: str, callback: EventCallback) -> EventSubscription: ...


class InMemoryGameEventBus:
    def __init__(self) -> None:
        self.published_events: list[tuple[str, dict[str, Any]]] = []
        self._subscribers: dict[str, set[EventCallback]] = defaultdict(set)

    async def publish(self, game_id: str, event: dict[str, Any]) -> None:
        self.published_events.append((game_id, dict(event)))
        for callback in list(self._subscribers.get(game_id, set())):
            await callback(dict(event))

    async def subscribe(self, game_id: str, callback: EventCallback) -> EventSubscription:
        self._subscribers[game_id].add(callback)
        return _InMemoryEventSubscription(self._subscribers[game_id], callback)


class _InMemoryEventSubscription:
    def __init__(self, subscribers: set[EventCallback], callback: EventCallback) -> None:
        self._subscribers = subscribers
        self._callback = callback

    async def close(self) -> None:
        self._subscribers.discard(self._callback)


class RedisGameEventBus:
    def __init__(self, url: str | None = None, redis_client: Any | None = None) -> None:
        redis_url = _redis_url(url)
        if redis_client is None:
            try:
                from redis.asyncio import Redis
            except ModuleNotFoundError as error:
                raise RuntimeError("Install the redis package to use RedisGameEventBus") from error

            redis_client = Redis.from_url(redis_url, decode_responses=True)

        self._redis: Any = redis_client

    async def publish(self, game_id: str, event: dict[str, Any]) -> None:
        payload = json.dumps(event)
        await self._redis.xadd(
            _stream_key(game_id),
            {"event": payload},
            maxlen=STREAM_MAXLEN,
            approximate=True,
        )
        await self._redis.publish(_channel_key(game_id), payload)

    async def subscribe(self, game_id: str, callback: EventCallback) -> EventSubscription:
        pubsub = self._redis.pubsub()
        await pubsub.subscribe(_channel_key(game_id))
        task = asyncio.create_task(_listen(pubsub, callback))
        return _RedisEventSubscription(pubsub, task, game_id)

    async def aclose(self) -> None:
        aclose = getattr(self._redis, "aclose", None)
        if aclose is not None:
            await aclose()


class _RedisEventSubscription:
    def __init__(self, pubsub: Any, task: asyncio.Task[None], game_id: str) -> None:
        self._pubsub = pubsub
        self._task = task
        self._game_id = game_id

    async def close(self) -> None:
        self._task.cancel()
        try:
            await self._task
        except asyncio.CancelledError:
            pass
        await self._pubsub.unsubscribe(_channel_key(self._game_id))
        await self._pubsub.close()


async def _listen(pubsub: Any, callback: EventCallback) -> None:
    while True:
        message = await pubsub.get_message(ignore_subscribe_messages=True, timeout=1.0)
        if message is None:
            await asyncio.sleep(0.01)
            continue

        data = message.get("data")
        if isinstance(data, bytes):
            data = data.decode()
        if not isinstance(data, str):
            continue

        try:
            event = json.loads(data)
        except json.JSONDecodeError:
            continue
        if isinstance(event, dict):
            await callback(event)


def _channel_key(game_id: str) -> str:
    return f"{GAME_EVENT_CHANNEL_PREFIX}{game_id}"


def _stream_key(game_id: str) -> str:
    return f"{GAME_EVENT_STREAM_PREFIX}{game_id}"


def _redis_url(url: str | None) -> str:
    if url is not None:
        return url
    return os.getenv("BLOCUS_REDIS_URL", "redis://localhost:6379/0")
