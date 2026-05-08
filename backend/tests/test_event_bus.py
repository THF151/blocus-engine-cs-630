from __future__ import annotations

import asyncio
import json
from typing import Any

import pytest

from blocus_backend.event_bus import (
    GAME_EVENT_CHANNEL_PREFIX,
    GAME_EVENT_STREAM_PREFIX,
    STREAM_MAXLEN,
    InMemoryGameEventBus,
    RedisGameEventBus,
)


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
