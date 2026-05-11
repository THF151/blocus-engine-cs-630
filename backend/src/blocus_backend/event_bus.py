from __future__ import annotations

import asyncio
import json
import logging
import os
from collections import defaultdict
from collections.abc import Awaitable, Callable
from typing import Any, Protocol

log = logging.getLogger(__name__)

GAME_EVENT_CHANNEL_PREFIX = "blocus:game_events:"
GAME_EVENT_STREAM_PREFIX = "blocus:game_stream:"
STREAM_MAXLEN = 1_000

EventCallback = Callable[[dict[str, Any]], Awaitable[None]]

_BACKOFF_INITIAL_SECONDS = 1.0
_BACKOFF_MAX_SECONDS = 30.0

try:
    from redis import (  # type: ignore[import-not-found,unused-ignore]
        RedisError as _RedisError,
    )

    _PUBSUB_ERROR_TYPES: tuple[type[BaseException], ...] = (
        _RedisError,
        ConnectionError,
        OSError,
        TimeoutError,
    )
except ModuleNotFoundError:
    _PUBSUB_ERROR_TYPES = (ConnectionError, OSError, TimeoutError)


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
        task = asyncio.create_task(_listen_with_recovery(self._redis, game_id, callback))
        return _RedisEventSubscription(task)

    async def aclose(self) -> None:
        aclose = getattr(self._redis, "aclose", None)
        if aclose is not None:
            await aclose()


class _RedisEventSubscription:
    def __init__(self, task: asyncio.Task[None]) -> None:
        self._task = task

    async def close(self) -> None:
        self._task.cancel()
        try:
            await self._task
        except asyncio.CancelledError:
            pass


async def _listen_with_recovery(
    redis_client: Any,
    game_id: str,
    callback: EventCallback,
) -> None:
    """Subscribe to the game's pubsub channel and dispatch messages.

    On Redis connection errors, log, back off exponentially (1, 2, 4, 8, 16,
    30s cap, retried indefinitely), re-subscribe, and replay the latest
    event from the stream so worker-local subscribers catch up.
    """
    backoff = _BACKOFF_INITIAL_SECONDS
    is_reconnect = False

    while True:
        pubsub = redis_client.pubsub()
        try:
            try:
                await pubsub.subscribe(_channel_key(game_id))
            except _PUBSUB_ERROR_TYPES as error:
                log.warning(
                    "pubsub subscribe failed for %s (%s); retrying in %.1fs",
                    game_id,
                    error,
                    backoff,
                )
                await asyncio.sleep(backoff)
                backoff = min(backoff * 2, _BACKOFF_MAX_SECONDS)
                continue

            if is_reconnect:
                await _replay_latest_event(redis_client, game_id, callback)
            backoff = _BACKOFF_INITIAL_SECONDS
            is_reconnect = True

            try:
                await _consume(pubsub, callback)
            except _PUBSUB_ERROR_TYPES as error:
                log.warning("pubsub disconnected for %s: %s; reconnecting", game_id, error)
        finally:
            await _close_silently(pubsub, game_id)


async def _consume(pubsub: Any, callback: EventCallback) -> None:
    while True:
        message = await pubsub.get_message(ignore_subscribe_messages=True, timeout=1.0)
        if message is None:
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


async def _replay_latest_event(
    redis_client: Any,
    game_id: str,
    callback: EventCallback,
) -> None:
    """Re-deliver the most recent event from the game's persistence stream.

    Called after a pubsub reconnect so worker-local subscribers receive the
    last broadcast they missed during the disconnect window and can re-render
    based on its embedded state view.
    """
    try:
        entries = await redis_client.xrevrange(_stream_key(game_id), count=1)
    except _PUBSUB_ERROR_TYPES as error:
        log.warning("stream replay failed for %s: %s", game_id, error)
        return
    if not entries:
        return

    _, fields = entries[0]
    raw = fields.get("event") if isinstance(fields, dict) else None
    if isinstance(raw, bytes):
        raw = raw.decode()
    if not isinstance(raw, str):
        return

    try:
        event = json.loads(raw)
    except json.JSONDecodeError:
        return
    if isinstance(event, dict):
        await callback(event)


async def _close_silently(pubsub: Any, game_id: str) -> None:
    try:
        await pubsub.unsubscribe(_channel_key(game_id))
    except Exception:
        log.debug("error unsubscribing pubsub for %s", game_id, exc_info=True)
    try:
        await pubsub.close()
    except Exception:
        log.debug("error closing pubsub for %s", game_id, exc_info=True)


def _channel_key(game_id: str) -> str:
    return f"{GAME_EVENT_CHANNEL_PREFIX}{game_id}"


def _stream_key(game_id: str) -> str:
    return f"{GAME_EVENT_STREAM_PREFIX}{game_id}"


def _redis_url(url: str | None) -> str:
    if url is not None:
        return url
    return os.getenv("BLOCUS_REDIS_URL", "redis://localhost:6379/0")
