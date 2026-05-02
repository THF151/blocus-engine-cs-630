from __future__ import annotations

import pytest

import blocus_engine


GAME_2 = "00000000-0000-0000-0000-000000000102"
GAME_3 = "00000000-0000-0000-0000-000000000103"
GAME_4 = "00000000-0000-0000-0000-000000000104"

PLAYER_1 = "00000000-0000-0000-0000-000000000001"
PLAYER_2 = "00000000-0000-0000-0000-000000000002"
PLAYER_3 = "00000000-0000-0000-0000-000000000003"
PLAYER_4 = "00000000-0000-0000-0000-000000000004"


def official_turn_order() -> list[blocus_engine.PlayerColor]:
    return [
        blocus_engine.PlayerColor.BLUE,
        blocus_engine.PlayerColor.YELLOW,
        blocus_engine.PlayerColor.RED,
        blocus_engine.PlayerColor.GREEN,
    ]


def custom_four_player_turn_order() -> list[blocus_engine.PlayerColor]:
    return [
        blocus_engine.PlayerColor.RED,
        blocus_engine.PlayerColor.GREEN,
        blocus_engine.PlayerColor.BLUE,
        blocus_engine.PlayerColor.YELLOW,
    ]


def test_player_slots_two_player_constructs_official_pairing() -> None:
    slots = blocus_engine.PlayerSlots.two_player(PLAYER_1, PLAYER_2)

    assert "PlayerSlots" in repr(slots)


def test_shared_color_turn_preserves_color_and_players() -> None:
    shared = blocus_engine.SharedColorTurn(
        blocus_engine.PlayerColor.GREEN,
        [PLAYER_1, PLAYER_2, PLAYER_3],
    )

    assert shared.color == blocus_engine.PlayerColor.GREEN
    assert shared.players == [PLAYER_1, PLAYER_2, PLAYER_3]
    assert "SharedColorTurn" in repr(shared)


def test_player_slots_three_player_accepts_owned_colors_and_shared_turn() -> None:
    shared = blocus_engine.SharedColorTurn(
        blocus_engine.PlayerColor.GREEN,
        [PLAYER_1, PLAYER_2, PLAYER_3],
    )
    slots = blocus_engine.PlayerSlots.three_player(
        [
            (blocus_engine.PlayerColor.BLUE, PLAYER_1),
            (blocus_engine.PlayerColor.YELLOW, PLAYER_2),
            (blocus_engine.PlayerColor.RED, PLAYER_3),
        ],
        shared,
    )

    assert "PlayerSlots" in repr(slots)


def test_player_slots_four_player_accepts_one_player_per_color() -> None:
    slots = blocus_engine.PlayerSlots.four_player(
        [
            (blocus_engine.PlayerColor.BLUE, PLAYER_1),
            (blocus_engine.PlayerColor.YELLOW, PLAYER_2),
            (blocus_engine.PlayerColor.RED, PLAYER_3),
            (blocus_engine.PlayerColor.GREEN, PLAYER_4),
        ]
    )

    assert "PlayerSlots" in repr(slots)


def test_game_config_two_player_convenience_constructor() -> None:
    config = blocus_engine.GameConfig.two_player(
        GAME_2,
        PLAYER_1,
        PLAYER_2,
        blocus_engine.ScoringMode.BASIC,
    )

    assert config.game_id == GAME_2
    assert config.mode == blocus_engine.GameMode.TWO_PLAYER
    assert config.scoring == blocus_engine.ScoringMode.BASIC
    assert [color.value for color in config.turn_order] == ["blue", "yellow", "red", "green"]
    assert "GameConfig" in repr(config)


def test_game_config_three_player_general_constructor() -> None:
    shared = blocus_engine.SharedColorTurn(
        blocus_engine.PlayerColor.GREEN,
        [PLAYER_1, PLAYER_2, PLAYER_3],
    )
    slots = blocus_engine.PlayerSlots.three_player(
        [
            (blocus_engine.PlayerColor.BLUE, PLAYER_1),
            (blocus_engine.PlayerColor.YELLOW, PLAYER_2),
            (blocus_engine.PlayerColor.RED, PLAYER_3),
        ],
        shared,
    )

    config = blocus_engine.GameConfig(
        GAME_3,
        blocus_engine.GameMode.THREE_PLAYER,
        blocus_engine.ScoringMode.ADVANCED,
        official_turn_order(),
        slots,
    )

    assert config.game_id == GAME_3
    assert config.mode == blocus_engine.GameMode.THREE_PLAYER
    assert config.scoring == blocus_engine.ScoringMode.ADVANCED
    assert [color.value for color in config.turn_order] == ["blue", "yellow", "red", "green"]


def test_game_config_four_player_convenience_constructor_with_custom_turn_order() -> None:
    config = blocus_engine.GameConfig.four_player(
        GAME_4,
        [
            (blocus_engine.PlayerColor.BLUE, PLAYER_1),
            (blocus_engine.PlayerColor.YELLOW, PLAYER_2),
            (blocus_engine.PlayerColor.RED, PLAYER_3),
            (blocus_engine.PlayerColor.GREEN, PLAYER_4),
        ],
        blocus_engine.ScoringMode.BASIC,
        custom_four_player_turn_order(),
    )

    assert config.game_id == GAME_4
    assert config.mode == blocus_engine.GameMode.FOUR_PLAYER
    assert [color.value for color in config.turn_order] == ["red", "green", "blue", "yellow"]


def test_initialize_two_player_game_exposes_stable_state_fields() -> None:
    engine = blocus_engine.BlocusEngine()
    config = blocus_engine.GameConfig.two_player(
        GAME_2,
        PLAYER_1,
        PLAYER_2,
        blocus_engine.ScoringMode.BASIC,
    )

    state = engine.initialize_game(config)

    assert state.schema_version == 1
    assert state.game_id == GAME_2
    assert state.mode == blocus_engine.GameMode.TWO_PLAYER
    assert state.scoring == blocus_engine.ScoringMode.BASIC
    assert state.status == blocus_engine.GameStatus.IN_PROGRESS
    assert state.version == 0
    assert state.hash != 0
    assert state.board_is_empty is True
    assert state.current_color == blocus_engine.PlayerColor.BLUE
    assert [color.value for color in state.turn_order] == ["blue", "yellow", "red", "green"]
    assert "GameState" in repr(state)


def test_initialize_three_player_game_exposes_shared_color_setup() -> None:
    engine = blocus_engine.BlocusEngine()
    shared = blocus_engine.SharedColorTurn(
        blocus_engine.PlayerColor.GREEN,
        [PLAYER_1, PLAYER_2, PLAYER_3],
    )
    slots = blocus_engine.PlayerSlots.three_player(
        [
            (blocus_engine.PlayerColor.BLUE, PLAYER_1),
            (blocus_engine.PlayerColor.YELLOW, PLAYER_2),
            (blocus_engine.PlayerColor.RED, PLAYER_3),
        ],
        shared,
    )
    config = blocus_engine.GameConfig(
        GAME_3,
        blocus_engine.GameMode.THREE_PLAYER,
        blocus_engine.ScoringMode.ADVANCED,
        official_turn_order(),
        slots,
    )

    state = engine.initialize_game(config)

    assert state.game_id == GAME_3
    assert state.mode == blocus_engine.GameMode.THREE_PLAYER
    assert state.scoring == blocus_engine.ScoringMode.ADVANCED
    assert state.current_color == blocus_engine.PlayerColor.BLUE
    assert state.board_is_empty is True


def test_initialize_four_player_game_uses_custom_turn_order_first_color() -> None:
    engine = blocus_engine.BlocusEngine()
    config = blocus_engine.GameConfig.four_player(
        GAME_4,
        [
            (blocus_engine.PlayerColor.BLUE, PLAYER_1),
            (blocus_engine.PlayerColor.YELLOW, PLAYER_2),
            (blocus_engine.PlayerColor.RED, PLAYER_3),
            (blocus_engine.PlayerColor.GREEN, PLAYER_4),
        ],
        blocus_engine.ScoringMode.BASIC,
        custom_four_player_turn_order(),
    )

    state = engine.initialize_game(config)

    assert state.game_id == GAME_4
    assert state.mode == blocus_engine.GameMode.FOUR_PLAYER
    assert state.current_color == blocus_engine.PlayerColor.RED
    assert [color.value for color in state.turn_order] == ["red", "green", "blue", "yellow"]


@pytest.mark.parametrize(
    "bad_players",
    [
        [PLAYER_1, PLAYER_2],
        [PLAYER_1, PLAYER_2, PLAYER_3, PLAYER_4],
    ],
)
def test_shared_color_turn_rejects_wrong_player_count(bad_players: list[str]) -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.SharedColorTurn(blocus_engine.PlayerColor.GREEN, bad_players)

    assert "input_error" in str(captured.value)
    assert "InvalidGameConfig" in str(captured.value)


def test_three_player_rejects_owned_shared_color_overlap() -> None:
    shared = blocus_engine.SharedColorTurn(
        blocus_engine.PlayerColor.GREEN,
        [PLAYER_1, PLAYER_2, PLAYER_3],
    )

    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PlayerSlots.three_player(
            [
                (blocus_engine.PlayerColor.BLUE, PLAYER_1),
                (blocus_engine.PlayerColor.YELLOW, PLAYER_2),
                (blocus_engine.PlayerColor.GREEN, PLAYER_3),
            ],
            shared,
        )

    assert "input_error" in str(captured.value)
    assert "InvalidGameConfig" in str(captured.value)


def test_four_player_rejects_duplicate_color_assignment() -> None:
    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.PlayerSlots.four_player(
            [
                (blocus_engine.PlayerColor.BLUE, PLAYER_1),
                (blocus_engine.PlayerColor.BLUE, PLAYER_2),
                (blocus_engine.PlayerColor.RED, PLAYER_3),
                (blocus_engine.PlayerColor.GREEN, PLAYER_4),
            ]
        )

    assert "input_error" in str(captured.value)
    assert "InvalidGameConfig" in str(captured.value)


def test_game_config_rejects_invalid_turn_order_shape() -> None:
    slots = blocus_engine.PlayerSlots.two_player(PLAYER_1, PLAYER_2)

    with pytest.raises(blocus_engine.InputError) as captured:
        blocus_engine.GameConfig(
            GAME_2,
            blocus_engine.GameMode.TWO_PLAYER,
            blocus_engine.ScoringMode.BASIC,
            [blocus_engine.PlayerColor.BLUE],
            slots,
        )

    assert "input_error" in str(captured.value)
    assert "InvalidGameConfig" in str(captured.value)
