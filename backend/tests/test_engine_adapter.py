from __future__ import annotations

import pytest

pytest.importorskip("blocus_engine")

from blocus_backend.engine_adapter import ClassicEngineAdapter, EngineUnavailableError  # noqa: E402

GAME_ID = "00000000-0000-0000-0000-000000009001"
PLAYER_ONE = "00000000-0000-0000-0000-000000000001"
PLAYER_TWO = "00000000-0000-0000-0000-000000000002"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def test_engine_adapter_creates_serializes_and_restores_classic_two_player_game() -> None:
    adapter = ClassicEngineAdapter()

    state = adapter.create_game(
        {
            "game_id": GAME_ID,
            "mode": "two_player",
            "scoring": "basic",
            "players": {
                "blue_red": PLAYER_ONE,
                "yellow_green": PLAYER_TWO,
            },
            "turn_order": ["blue", "yellow", "red", "green"],
        }
    )
    restored = adapter.deserialize_state(adapter.serialize_state(state))
    view = adapter.state_view(restored)

    assert view["game_id"] == GAME_ID
    assert view["mode"] == "two_player"
    assert view["board_size"] == 20
    assert view["current_color"] == "blue"
    assert view["turn_order"] == ["blue", "yellow", "red", "green"]


def test_engine_adapter_applies_first_legal_classic_move() -> None:
    adapter = ClassicEngineAdapter()
    state = adapter.create_game(
        {
            "game_id": GAME_ID,
            "mode": "two_player",
            "scoring": "basic",
            "players": {
                "blue_red": PLAYER_ONE,
                "yellow_green": PLAYER_TWO,
            },
            "turn_order": ["blue", "yellow", "red", "green"],
        }
    )
    move = adapter.legal_moves(state, PLAYER_ONE, "blue")[0]

    result = adapter.place_move(
        state,
        {
            "game_id": GAME_ID,
            "command_id": uuid(1),
            "player_id": PLAYER_ONE,
            "color": "blue",
            **move,
        },
    )

    assert result.event_type == "move_applied"
    assert result.response == "move applied"
    assert adapter.state_view(result.next_state)["version"] == 1
    assert adapter.state_view(result.next_state)["current_color"] == "yellow"


def test_engine_adapter_creates_three_player_shared_green_game() -> None:
    adapter = ClassicEngineAdapter()

    state = adapter.create_game(
        {
            "game_id": GAME_ID,
            "mode": "three_player",
            "scoring": "advanced",
            "players": {
                "blue": PLAYER_ONE,
                "yellow": PLAYER_TWO,
                "red": "00000000-0000-0000-0000-000000000003",
                "shared_green": [PLAYER_ONE, PLAYER_TWO, "00000000-0000-0000-0000-000000000003"],
            },
            "turn_order": ["blue", "yellow", "red", "green"],
        }
    )

    view = adapter.state_view(state)

    assert view["mode"] == "three_player"
    assert view["scoring"] == "advanced"
    assert view["turn_order"] == ["blue", "yellow", "red", "green"]
    assert view["shared_color_turn_index"] == 0


def test_engine_adapter_creates_four_player_game_with_rotated_order() -> None:
    adapter = ClassicEngineAdapter()

    state = adapter.create_game(
        {
            "game_id": GAME_ID,
            "mode": "four_player",
            "scoring": "basic",
            "players": {
                "blue": PLAYER_ONE,
                "yellow": PLAYER_TWO,
                "red": "00000000-0000-0000-0000-000000000003",
                "green": "00000000-0000-0000-0000-000000000004",
            },
            "turn_order": ["red", "green", "blue", "yellow"],
        }
    )

    view = adapter.state_view(state)

    assert view["mode"] == "four_player"
    assert view["current_color"] == "red"
    assert view["turn_order"] == ["red", "green", "blue", "yellow"]


def test_engine_adapter_rejects_unknown_module_when_binding_missing(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    adapter = ClassicEngineAdapter()

    def raise_missing() -> None:
        raise EngineUnavailableError("missing")

    monkeypatch.setattr(adapter, "_module", raise_missing)

    assert adapter.engine_health() is False


def test_engine_adapter_creates_duo_game_with_14x14_board() -> None:
    adapter = ClassicEngineAdapter()

    state = adapter.create_game(
        {
            "game_id": GAME_ID,
            "mode": "duo",
            "scoring": "advanced",
            "players": {"black": PLAYER_ONE, "white": PLAYER_TWO},
            "turn_order": ["black", "white"],
        }
    )

    view = adapter.state_view(state)

    assert view["mode"] == "duo"
    assert view["board_size"] == 14
    assert view["scoring"] == "advanced"
    assert view["current_color"] == "black"
    assert view["turn_order"] == ["black", "white"]


def test_engine_adapter_creates_duo_game_starting_with_white() -> None:
    adapter = ClassicEngineAdapter()

    state = adapter.create_game(
        {
            "game_id": GAME_ID,
            "mode": "duo",
            "scoring": "advanced",
            "players": {"black": PLAYER_ONE, "white": PLAYER_TWO},
            "turn_order": ["white", "black"],
        }
    )

    view = adapter.state_view(state)

    assert view["current_color"] == "white"
    assert view["turn_order"] == ["white", "black"]


def test_engine_adapter_rejects_unknown_mode() -> None:
    adapter = ClassicEngineAdapter()

    with pytest.raises(ValueError, match="Unsupported mode"):
        adapter.create_game(
            {
                "game_id": GAME_ID,
                "mode": "custom",
                "scoring": "basic",
                "players": {},
                "turn_order": ["blue", "yellow", "red", "green"],
            }
        )
