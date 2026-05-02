from __future__ import annotations

import json

import blocus_engine as be


GAME_ID = "00000000-0000-0000-0000-000000000700"
PLAYER_1 = "00000000-0000-0000-0000-000000000001"
PLAYER_2 = "00000000-0000-0000-0000-000000000002"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def initialized_state() -> tuple[be.BlocusEngine, be.GameState]:
    engine = be.BlocusEngine()
    config = be.GameConfig.two_player(
        GAME_ID,
        PLAYER_1,
        PLAYER_2,
        be.ScoringMode.BASIC,
    )

    return engine, engine.initialize_game(config)


def test_game_state_to_json_contains_stable_public_state() -> None:
    _, state = initialized_state()

    text = state.to_json()
    data = json.loads(text)

    assert data["schema_version"] == 1
    assert data["game_id"] == GAME_ID
    assert data["mode"] == "two_player"
    assert data["scoring"] == "basic"
    assert data["status"] == "in_progress"
    assert data["version"] == 0
    assert isinstance(data["hash"], int)
    assert data["hash"] != 0
    assert data["turn"]["current_color"] == "blue"
    assert data["turn_order"] == ["blue", "yellow", "red", "green"]
    assert len(data["board"]) == 4
    assert len(data["inventories"]) == 4


def test_game_state_json_round_trip_preserves_initial_state_fields() -> None:
    _, state = initialized_state()

    restored = be.GameState.from_json(state.to_json())

    assert restored.schema_version == state.schema_version
    assert restored.game_id == state.game_id
    assert restored.mode == state.mode
    assert restored.scoring == state.scoring
    assert restored.status == state.status
    assert restored.version == state.version
    assert restored.hash == state.hash
    assert restored.board_is_empty == state.board_is_empty
    assert restored.current_color == state.current_color
    assert [color.value for color in restored.turn_order] == [
        color.value for color in state.turn_order
    ]


def test_game_state_json_round_trip_preserves_state_after_move() -> None:
    engine, state = initialized_state()

    command = be.PlaceCommand(
        command_id=uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_1,
        color=be.PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )

    result = engine.apply(state, command)
    next_state = result.next_state

    restored = be.GameState.from_json(next_state.to_json())

    assert restored.game_id == next_state.game_id
    assert restored.status == next_state.status
    assert restored.version == next_state.version
    assert restored.hash == next_state.hash
    assert restored.board_is_empty is False
    assert restored.current_color == be.PlayerColor.YELLOW

    yellow_moves = engine.get_valid_moves(
        restored,
        PLAYER_2,
        be.PlayerColor.YELLOW,
    )

    assert yellow_moves
    assert yellow_moves[0].row == 0
    assert yellow_moves[0].col == 19