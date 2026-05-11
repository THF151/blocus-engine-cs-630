from __future__ import annotations

import asyncio
import json
from collections import defaultdict
from typing import Any

import pytest

from blocus_backend.event_bus import (
    GAME_EVENT_CHANNEL_PREFIX,
    GAME_EVENT_STREAM_PREFIX,
    STREAM_MAXLEN,
    InMemoryGameEventBus,
    RedisGameEventBus,
)

_DISCONNECT_SENTINEL = "__FAIL__"


class FakeRedisClient:
    def __init__(self) -> None:
        self.xadd_calls: list[dict[str, Any]] = []
        self.publish_calls: list[dict[str, Any]] = []

    async def xadd(
        self,
        key: str,
        fields: dict[str, str],
        *,
        maxlen: int,
        approximate: bool,
    ) -> None:
        self.xadd_calls.append(
            {
                "key": key,
                "fields": fields,
                "maxlen": maxlen,
                "approximate": approximate,
            }
        )

    async def publish(self, channel: str, payload: str) -> None:
        self.publish_calls.append({"channel": channel, "payload": payload})


class FakeRedisClientWithPubSub(FakeRedisClient):
    def __init__(self, pubsub: FakePubSub) -> None:
        super().__init__()
        self._pubsub = pubsub

    def pubsub(self) -> FakePubSub:
        return self._pubsub


class FakePubSub:
    def __init__(self) -> None:
        self.messages: asyncio.Queue[dict[str, Any] | None] = asyncio.Queue()
        self.subscribed_channels: list[str] = []
        self.unsubscribed_channels: list[str] = []
        self.closed = False

    async def subscribe(self, channel: str) -> None:
        self.subscribed_channels.append(channel)

    async def get_message(
        self,
        *,
        ignore_subscribe_messages: bool,
        timeout: float,
    ) -> dict[str, Any] | None:
        del ignore_subscribe_messages, timeout
        return await self.messages.get()

    async def unsubscribe(self, channel: str) -> None:
        self.unsubscribed_channels.append(channel)

    async def close(self) -> None:
        self.closed = True


@pytest.mark.asyncio
async def test_in_memory_event_bus_fans_out_to_subscribers() -> None:
    bus = InMemoryGameEventBus()
    received: list[dict[str, Any]] = []

    async def callback(event: dict[str, Any]) -> None:
        received.append(event)

    subscription = await bus.subscribe("game-1", callback)

    await bus.publish("game-1", {"type": "move_applied", "game_id": "game-1"})
    await subscription.close()
    await bus.publish("game-1", {"type": "move_applied", "game_id": "game-1"})

    assert received == [{"type": "move_applied", "game_id": "game-1"}]


@pytest.mark.asyncio
async def test_redis_event_bus_writes_stream_and_publishes_channel() -> None:
    redis = FakeRedisClient()
    bus = RedisGameEventBus(url="redis://custom", redis_client=redis)
    event = {"type": "move_applied", "game_id": "game-redis", "state": {"version": 1}}

    await bus.publish("game-redis", event)

    expected_payload = json.dumps(event)
    assert redis.xadd_calls == [
        {
            "key": f"{GAME_EVENT_STREAM_PREFIX}game-redis",
            "fields": {"event": expected_payload},
            "maxlen": STREAM_MAXLEN,
            "approximate": True,
        }
    ]
    assert redis.publish_calls == [
        {
            "channel": f"{GAME_EVENT_CHANNEL_PREFIX}game-redis",
            "payload": expected_payload,
        }
    ]


@pytest.mark.asyncio
async def test_redis_event_bus_subscribes_and_dispatches_pubsub_messages() -> None:
    pubsub = FakePubSub()
    redis = FakeRedisClientWithPubSub(pubsub)
    bus = RedisGameEventBus(redis_client=redis)
    received: list[dict[str, Any]] = []

    async def callback(event: dict[str, Any]) -> None:
        received.append(event)

    subscription = await bus.subscribe("game-redis", callback)
    await pubsub.messages.put(None)
    await pubsub.messages.put({"data": b"not-json"})
    await pubsub.messages.put({"data": 42})
    await pubsub.messages.put({"data": json.dumps({"type": "move_applied"})})

    for _ in range(20):
        if received:
            break
        await asyncio.sleep(0.01)

    await subscription.close()

    assert pubsub.subscribed_channels == [f"{GAME_EVENT_CHANNEL_PREFIX}game-redis"]
    assert pubsub.unsubscribed_channels == [f"{GAME_EVENT_CHANNEL_PREFIX}game-redis"]
    assert pubsub.closed is True
    assert received == [{"type": "move_applied"}]


class _RecoveryPubSub:
    """Pubsub fake that raises ConnectionError when a sentinel value is read.

    Each instance represents a single connection lifetime — the recovery
    loop creates a new ``_RecoveryPubSub`` after every disconnect.
    """

    def __init__(self) -> None:
        self.messages: asyncio.Queue[Any] = asyncio.Queue()
        self.subscribed_channels: list[str] = []
        self.unsubscribed_channels: list[str] = []
        self.closed = False

    async def subscribe(self, channel: str) -> None:
        self.subscribed_channels.append(channel)

    async def get_message(
        self,
        *,
        ignore_subscribe_messages: bool,
        timeout: float,
    ) -> dict[str, Any] | None:
        del ignore_subscribe_messages, timeout
        item = await self.messages.get()
        if item == _DISCONNECT_SENTINEL:
            raise ConnectionError("simulated pubsub disconnect")
        return item

    async def unsubscribe(self, channel: str) -> None:
        self.unsubscribed_channels.append(channel)

    async def close(self) -> None:
        self.closed = True


class _RecoveryRedis:
    """Redis fake with stream persistence (xrevrange) and a pubsub factory
    that hands out a fresh ``_RecoveryPubSub`` each call."""

    def __init__(self) -> None:
        self.streams: dict[str, list[dict[str, str]]] = defaultdict(list)
        self.pubsub_instances: list[_RecoveryPubSub] = []

    async def xadd(
        self,
        key: str,
        fields: dict[str, str],
        *,
        maxlen: int,
        approximate: bool,
    ) -> None:
        del maxlen, approximate
        self.streams[key].append(dict(fields))

    async def publish(self, channel: str, payload: str) -> None:
        del channel, payload

    async def xrevrange(self, key: str, count: int) -> list[tuple[str, dict[str, str]]]:
        entries = self.streams.get(key, [])
        latest = list(reversed(entries))[:count]
        return [(f"{i}-0", dict(fields)) for i, fields in enumerate(latest)]

    def pubsub(self) -> _RecoveryPubSub:
        ps = _RecoveryPubSub()
        self.pubsub_instances.append(ps)
        return ps


async def _wait_until(predicate: Any, *, attempts: int = 50, delay: float = 0.01) -> None:
    for _ in range(attempts):
        if predicate():
            return
        await asyncio.sleep(delay)


@pytest.mark.asyncio
async def test_redis_event_bus_reconnects_after_disconnect(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr("blocus_backend.event_bus._BACKOFF_INITIAL_SECONDS", 0.0)
    monkeypatch.setattr("blocus_backend.event_bus._BACKOFF_MAX_SECONDS", 0.0)

    redis = _RecoveryRedis()
    bus = RedisGameEventBus(redis_client=redis)

    received: list[dict[str, Any]] = []

    async def callback(event: dict[str, Any]) -> None:
        received.append(event)

    subscription = await bus.subscribe("game-r", callback)
    await _wait_until(lambda: len(redis.pubsub_instances) >= 1)

    first = redis.pubsub_instances[0]
    await first.messages.put(_DISCONNECT_SENTINEL)

    await _wait_until(lambda: len(redis.pubsub_instances) >= 2)

    await subscription.close()

    assert len(redis.pubsub_instances) >= 2
    assert first.closed is True
    assert first.unsubscribed_channels == [f"{GAME_EVENT_CHANNEL_PREFIX}game-r"]


@pytest.mark.asyncio
async def test_redis_event_bus_replays_latest_event_after_reconnect(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr("blocus_backend.event_bus._BACKOFF_INITIAL_SECONDS", 0.0)
    monkeypatch.setattr("blocus_backend.event_bus._BACKOFF_MAX_SECONDS", 0.0)

    redis = _RecoveryRedis()
    bus = RedisGameEventBus(redis_client=redis)

    # Seed the stream with the "last event" before subscribing
    await bus.publish("game-r", {"type": "move_applied", "state": {"version": 7}})

    received: list[dict[str, Any]] = []

    async def callback(event: dict[str, Any]) -> None:
        received.append(event)

    subscription = await bus.subscribe("game-r", callback)
    await _wait_until(lambda: len(redis.pubsub_instances) >= 1)

    first = redis.pubsub_instances[0]
    await first.messages.put(_DISCONNECT_SENTINEL)

    await _wait_until(lambda: len(received) >= 1)
    await subscription.close()

    assert received[0] == {"type": "move_applied", "state": {"version": 7}}


@pytest.mark.asyncio
async def test_redis_event_bus_does_not_replay_on_initial_subscribe(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Initial subscribe must not replay — the client is already up-to-date."""
    monkeypatch.setattr("blocus_backend.event_bus._BACKOFF_INITIAL_SECONDS", 0.0)
    monkeypatch.setattr("blocus_backend.event_bus._BACKOFF_MAX_SECONDS", 0.0)

    redis = _RecoveryRedis()
    bus = RedisGameEventBus(redis_client=redis)

    await bus.publish("game-r", {"type": "move_applied", "state": {"version": 7}})

    received: list[dict[str, Any]] = []

    async def callback(event: dict[str, Any]) -> None:
        received.append(event)

    subscription = await bus.subscribe("game-r", callback)
    # Give the loop time to enter _consume
    await asyncio.sleep(0.05)

    await subscription.close()

    assert received == []
