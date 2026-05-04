from __future__ import annotations

import json

import pytest

import blocus_engine as be


GAME_ID = "00000000-0000-0000-0000-000000000900"
BLACK_PLAYER = "00000000-0000-0000-0000-000000000001"
WHITE_PLAYER = "00000000-0000-0000-0000-000000000002"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def duo_state() -> tuple[be.BlocusEngine, be.GameState]:
    engine = be.BlocusEngine()
    config = be.GameConfig.duo(GAME_ID, BLACK_PLAYER, WHITE_PLAYER)
    return engine, engine.initialize_game(config)


def place(
    *,
    command_id: int,
    color: be.PlayerColor,
    player_id: str,
    piece_id: int,
    row: int,
    col: int,
) -> be.PlaceCommand:
    return be.PlaceCommand(
        command_id=uuid(command_id),
        game_id=GAME_ID,
        player_id=player_id,
        color=color,
        piece_id=piece_id,
        orientation_id=0,
        row=row,
        col=col,
    )


def test_duo_enums_and_config_initialize_black_white_board() -> None:
    assert be.PlayerColor.BLACK == be.PlayerColor.black()
    assert be.PlayerColor.WHITE == be.PlayerColor("white")
    assert be.GameMode.DUO == be.GameMode.duo()

    config = be.GameConfig.duo(GAME_ID, BLACK_PLAYER, WHITE_PLAYER)
    engine = be.BlocusEngine()
    state = engine.initialize_game(config)

    assert config.mode == be.GameMode.DUO
    assert config.scoring == be.ScoringMode.ADVANCED
    assert [color.value for color in config.turn_order] == ["black", "white"]
    assert state.board_size == 14
    assert len(state.board_matrix()) == 14
    assert len(state.board_matrix()[0]) == 14
    assert state.current_color == be.PlayerColor.BLACK
    assert dict(state.board_counts()) == {
        be.PlayerColor.BLACK: 0,
        be.PlayerColor.WHITE: 0,
    }


def test_duo_rejects_basic_scoring() -> None:
    with pytest.raises(be.InputError) as captured:
        be.GameConfig.duo(
            GAME_ID,
            BLACK_PLAYER,
            WHITE_PLAYER,
            scoring=be.ScoringMode.BASIC,
        )

    assert "InvalidGameConfig" in str(captured.value)


def test_duo_opening_moves_cover_starting_points_and_second_start_remains() -> None:
    engine, state = duo_state()
    starts = {(4, 4), (9, 9)}

    initial_moves = engine.get_valid_moves(state, BLACK_PLAYER, be.PlayerColor.BLACK)
    assert initial_moves
    assert all((move.row, move.col) in starts for move in initial_moves if move.piece_id == 0)

    state = engine.apply(
        state,
        place(
            command_id=1,
            color=be.PlayerColor.BLACK,
            player_id=BLACK_PLAYER,
            piece_id=0,
            row=4,
            col=4,
        ),
    ).next_state

    white_moves = engine.get_valid_moves(state, WHITE_PLAYER, be.PlayerColor.WHITE)
    assert white_moves
    assert all((move.row, move.col) == (9, 9) for move in white_moves if move.piece_id == 0)

    with pytest.raises(be.RuleViolationError) as captured:
        engine.apply(
            state,
            place(
                command_id=2,
                color=be.PlayerColor.WHITE,
                player_id=WHITE_PLAYER,
                piece_id=0,
                row=0,
                col=0,
            ),
        )

    assert "MissingCornerContact" in str(captured.value)


def test_duo_json_round_trip_is_mode_aware() -> None:
    engine, state = duo_state()
    state = engine.apply(
        state,
        place(
            command_id=1,
            color=be.PlayerColor.BLACK,
            player_id=BLACK_PLAYER,
            piece_id=0,
            row=4,
            col=4,
        ),
    ).next_state

    data = json.loads(state.to_json())
    assert data["mode"] == "duo"
    assert data["scoring"] == "advanced"
    assert data["turn_order"] == ["black", "white"]
    assert set(data["board"]) == {"black", "white"}
    assert len(data["inventories"]) == 2

    restored = be.GameState.from_json(state.to_json())
    assert restored.mode == be.GameMode.DUO
    assert restored.board_size == 14
    assert restored.hash == state.hash
    assert restored.cell(4, 4) == be.PlayerColor.BLACK


def test_duo_rejects_outside_fourteen_by_fourteen_board() -> None:
    engine, state = duo_state()

    with pytest.raises(be.RuleViolationError) as captured:
        engine.apply(
            state,
            place(
                command_id=1,
                color=be.PlayerColor.BLACK,
                player_id=BLACK_PLAYER,
                piece_id=0,
                row=14,
                col=14,
            ),
        )

    assert "OutOfBounds" in str(captured.value)

    with pytest.raises(be.InputError):
        state.cell(14, 0)
