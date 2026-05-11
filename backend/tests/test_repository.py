from __future__ import annotations

import pytest
from fakeredis import FakeAsyncRedis

from blocus_backend.repository import (
    GameNotFoundError,
    InMemoryGameRepository,
    OptimisticLockError,
    RedisGameRepository,
)


@pytest.mark.asyncio
async def test_inmemory_save_game_unconditional_when_expected_version_is_none() -> None:
    repo = InMemoryGameRepository()

    await repo.save_game("g", "{}", {"a": 1}, expected_version=None)
    record = await repo.get_game("g")

    assert record.version == 1
    assert record.state_json == "{}"


@pytest.mark.asyncio
async def test_inmemory_save_game_bumps_version_on_each_save() -> None:
    repo = InMemoryGameRepository()
    await repo.save_game("g", "{}", {}, expected_version=None)

    await repo.save_game("g", '{"v": 1}', {}, expected_version=1)
    await repo.save_game("g", '{"v": 2}', {}, expected_version=2)

    record = await repo.get_game("g")
    assert record.version == 3
    assert record.state_json == '{"v": 2}'


@pytest.mark.asyncio
async def test_inmemory_save_game_raises_optimistic_lock_error_on_version_mismatch() -> None:
    repo = InMemoryGameRepository()
    await repo.save_game("g", "{}", {}, expected_version=None)

    with pytest.raises(OptimisticLockError) as captured:
        await repo.save_game("g", "{}", {}, expected_version=99)

    assert captured.value.game_id == "g"
    assert captured.value.expected_version == 99


@pytest.mark.asyncio
async def test_inmemory_save_game_unconditional_on_existing_record_still_bumps() -> None:
    """expected_version=None on an existing record bumps the version monotonically."""
    repo = InMemoryGameRepository()
    await repo.save_game("g", "{}", {}, expected_version=None)
    await repo.save_game("g", "{}", {}, expected_version=1)

    await repo.save_game("g", '{"forced": true}', {}, expected_version=None)

    record = await repo.get_game("g")
    assert record.version == 3
    assert record.state_json == '{"forced": true}'


@pytest.mark.asyncio
async def test_inmemory_get_game_raises_when_missing() -> None:
    repo = InMemoryGameRepository()

    with pytest.raises(GameNotFoundError):
        await repo.get_game("missing")


def _redis_repo() -> RedisGameRepository:
    return RedisGameRepository(redis_client=FakeAsyncRedis(decode_responses=True))


@pytest.mark.asyncio
async def test_redis_save_game_creates_record_unconditionally() -> None:
    repo = _redis_repo()

    await repo.save_game("game-x", '{"foo": 1}', {"mode": "two_player"}, expected_version=None)
    record = await repo.get_game("game-x")

    assert record.version == 1
    assert record.state_json == '{"foo": 1}'
    assert record.metadata == {"mode": "two_player"}


@pytest.mark.asyncio
async def test_redis_save_game_with_matching_expected_version_bumps_version() -> None:
    repo = _redis_repo()
    await repo.save_game("g", "{}", {}, expected_version=None)

    await repo.save_game("g", '{"v": 1}', {}, expected_version=1)
    await repo.save_game("g", '{"v": 2}', {}, expected_version=2)

    record = await repo.get_game("g")
    assert record.version == 3
    assert record.state_json == '{"v": 2}'


@pytest.mark.asyncio
async def test_redis_save_game_with_stale_expected_version_raises() -> None:
    repo = _redis_repo()
    await repo.save_game("g", "{}", {}, expected_version=None)

    with pytest.raises(OptimisticLockError) as captured:
        await repo.save_game("g", '{"v": 1}', {}, expected_version=99)

    assert captured.value.game_id == "g"
    assert captured.value.expected_version == 99


@pytest.mark.asyncio
async def test_redis_save_game_unconditional_on_existing_record_still_bumps() -> None:
    repo = _redis_repo()
    await repo.save_game("g", "{}", {}, expected_version=None)
    await repo.save_game("g", "{}", {}, expected_version=1)

    await repo.save_game("g", '{"forced": true}', {}, expected_version=None)

    record = await repo.get_game("g")
    assert record.version == 3
    assert record.state_json == '{"forced": true}'


@pytest.mark.asyncio
async def test_redis_save_game_with_expected_version_on_missing_record_raises() -> None:
    """expected_version on a never-saved game must fail; only None can create."""
    repo = _redis_repo()

    with pytest.raises(OptimisticLockError):
        await repo.save_game("never-saved", "{}", {}, expected_version=0)


@pytest.mark.asyncio
async def test_redis_save_game_concurrent_writers_only_one_wins() -> None:
    """Both writers read at version=1, both attempt expected_version=1; one wins."""
    repo = _redis_repo()
    await repo.save_game("g", "{}", {}, expected_version=None)  # version becomes 1

    await repo.save_game("g", '{"writer": "A"}', {}, expected_version=1)

    with pytest.raises(OptimisticLockError):
        await repo.save_game("g", '{"writer": "B"}', {}, expected_version=1)

    record = await repo.get_game("g")
    assert record.version == 2
    assert record.state_json == '{"writer": "A"}'


@pytest.mark.asyncio
async def test_redis_get_game_raises_when_missing() -> None:
    repo = _redis_repo()

    with pytest.raises(GameNotFoundError):
        await repo.get_game("missing")
