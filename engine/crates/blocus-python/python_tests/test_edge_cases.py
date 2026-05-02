from __future__ import annotations

import json

import pytest

import blocus_engine as be


GAME_ID = "00000000-0000-0000-0000-000000000900"
PLAYER_1 = "00000000-0000-0000-0000-000000000001"
PLAYER_2 = "00000000-0000-0000-0000-000000000002"
PLAYER_3 = "00000000-0000-0000-0000-000000000003"
PLAYER_4 = "00000000-0000-0000-0000-000000000004"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def official_turn_order() -> list[be.PlayerColor]:
    return [
        be.PlayerColor.BLUE,
        be.PlayerColor.YELLOW,
        be.PlayerColor.RED,
        be.PlayerColor.GREEN,
    ]


def three_player_state() -> tuple[be.BlocusEngine, be.GameState]:
    engine = be.BlocusEngine()
    shared = be.SharedColorTurn(
        be.PlayerColor.GREEN,
        [PLAYER_1, PLAYER_2, PLAYER_3],
    )
    slots = be.PlayerSlots.three_player(
        [
            (be.PlayerColor.BLUE, PLAYER_1),
            (be.PlayerColor.YELLOW, PLAYER_2),
            (be.PlayerColor.RED, PLAYER_3),
        ],
        shared,
    )
    config = be.GameConfig(
        GAME_ID,
        be.GameMode.THREE_PLAYER,
        be.ScoringMode.BASIC,
        official_turn_order(),
        slots,
    )

    return engine, engine.initialize_game(config)


def two_player_state() -> tuple[be.BlocusEngine, be.GameState]:
    engine = be.BlocusEngine()
    config = be.GameConfig.two_player(
        GAME_ID,
        PLAYER_1,
        PLAYER_2,
        be.ScoringMode.BASIC,
    )

    return engine, engine.initialize_game(config)


def state_with_turn(
    state: be.GameState,
    current_color: str,
    shared_color_turn_index: int = 0,
    passed_mask: int = 0,
) -> be.GameState:
    data = json.loads(state.to_json())
    data["turn"]["current_color"] = current_color
    data["turn"]["shared_color_turn_index"] = shared_color_turn_index
    data["turn"]["passed_mask"] = passed_mask

    return be.GameState.from_json(json.dumps(data))


def test_three_player_shared_color_scheduled_ownership_is_exposed_by_state_transition() -> None:
    engine, state = three_player_state()
    state = state_with_turn(state, "green", shared_color_turn_index=1)

    command = be.PlaceCommand(
        command_id=uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_2,
        color=be.PlayerColor.GREEN,
        piece_id=0,
        orientation_id=0,
        row=19,
        col=0,
    )

    result = engine.apply(state, command)

    assert result.next_state.current_color == be.PlayerColor.BLUE
    assert result.next_state.version == state.version + 1
    assert result.response.kind == be.DomainResponseKind.MOVE_APPLIED


def test_illegal_shared_color_move_by_wrong_shared_player_is_rejected() -> None:
    engine, state = three_player_state()
    state = state_with_turn(state, "green", shared_color_turn_index=1)

    command = be.PlaceCommand(
        command_id=uuid(2),
        game_id=GAME_ID,
        player_id=PLAYER_1,
        color=be.PlayerColor.GREEN,
        piece_id=0,
        orientation_id=0,
        row=19,
        col=0,
    )

    with pytest.raises(be.RuleViolationError) as captured:
        engine.apply(state, command)

    assert "PlayerDoesNotControlColor" in str(captured.value)


def test_legal_shared_color_move_by_correct_shared_player_is_accepted() -> None:
    engine, state = three_player_state()
    state = state_with_turn(state, "green", shared_color_turn_index=2)

    command = be.PlaceCommand(
        command_id=uuid(3),
        game_id=GAME_ID,
        player_id=PLAYER_3,
        color=be.PlayerColor.GREEN,
        piece_id=0,
        orientation_id=0,
        row=19,
        col=0,
    )

    result = engine.apply(state, command)

    assert result.next_state.version == state.version + 1
    assert result.next_state.board_is_empty is False
    assert [event.kind for event in result.events] == [
        be.DomainEventKind.MOVE_APPLIED,
        be.DomainEventKind.TURN_ADVANCED,
    ]


def test_pass_hash_is_recomputed_after_successful_pass() -> None:
    engine, state = two_player_state()

    data = json.loads(state.to_json())
    data["board"]["yellow"]["lanes"] = ["4294967299", "0", "0", "0", "0"]
    data["board"]["yellow"]["count"] = 4

    blocked_blue = be.GameState.from_json(json.dumps(data))

    command = be.PassCommand(
        command_id=uuid(4),
        game_id=GAME_ID,
        player_id=PLAYER_1,
        color=be.PlayerColor.BLUE,
    )

    result = engine.apply(blocked_blue, command)

    assert result.next_state.version == blocked_blue.version + 1
    assert result.next_state.hash != 0
    assert result.next_state.hash == be.GameState.from_json(result.next_state.to_json()).hash


def test_four_player_rejects_illegal_non_clockwise_permutation() -> None:
    with pytest.raises(be.InputError) as captured:
        be.GameConfig.four_player(
            GAME_ID,
            [
                (be.PlayerColor.BLUE, PLAYER_1),
                (be.PlayerColor.YELLOW, PLAYER_2),
                (be.PlayerColor.RED, PLAYER_3),
                (be.PlayerColor.GREEN, PLAYER_4),
            ],
            be.ScoringMode.BASIC,
            [
                be.PlayerColor.BLUE,
                be.PlayerColor.RED,
                be.PlayerColor.YELLOW,
                be.PlayerColor.GREEN,
            ],
        )

    assert "InvalidGameConfig" in str(captured.value)


def test_json_board_masks_with_padding_bits_are_rejected() -> None:
    _, state = two_player_state()
    data = json.loads(state.to_json())

    data["board"]["blue"]["lanes"] = [str(1 << 20), "0", "0", "0", "0"]
    data["board"]["blue"]["count"] = 1

    with pytest.raises(be.EngineError) as captured:
        be.GameState.from_json(json.dumps(data))

    assert "CorruptedState" in str(captured.value)


def test_json_board_masks_with_overlapping_colors_are_rejected() -> None:
    _, state = two_player_state()
    data = json.loads(state.to_json())

    data["board"]["blue"]["lanes"] = ["1", "0", "0", "0", "0"]
    data["board"]["blue"]["count"] = 1
    data["board"]["yellow"]["lanes"] = ["1", "0", "0", "0", "0"]
    data["board"]["yellow"]["count"] = 1

    with pytest.raises(be.EngineError) as captured:
        be.GameState.from_json(json.dumps(data))

    assert "CorruptedState" in str(captured.value)