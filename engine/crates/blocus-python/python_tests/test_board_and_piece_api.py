import json
import uuid

import pytest

from blocus_engine import (
    BlocusEngine,
    EngineError,
    GameConfig,
    GameMode,
    GameState,
    InputError,
    PlaceCommand,
    PlayerColor,
    PlayerSlots,
    ScoringMode,
)


BLUE_PLAYER = "00000000-0000-0000-0000-00000000000a"
YELLOW_GREEN_PLAYER = "00000000-0000-0000-0000-000000000014"
GAME_ID = "00000000-0000-0000-0000-000000000001"


def make_config():
    return GameConfig(
        game_id=GAME_ID,
        mode=GameMode.TWO_PLAYER,
        scoring=ScoringMode.BASIC,
        turn_order=[
            PlayerColor.BLUE,
            PlayerColor.YELLOW,
            PlayerColor.RED,
            PlayerColor.GREEN,
        ],
        player_slots=PlayerSlots.two_player(
            BLUE_PLAYER,
            YELLOW_GREEN_PLAYER,
        ),
    )


def make_place_command(state):
    return PlaceCommand(
        command_id=str(uuid.uuid4()),
        game_id=state.game_id,
        player_id=BLUE_PLAYER,
        color=PlayerColor.BLUE,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=0,
    )


def make_state():
    return BlocusEngine().initialize_game(make_config())


def test_game_state_exposes_empty_board_views():
    state = make_state()

    assert state.board_is_empty is True
    assert state.cell(0, 0) is None
    assert state.occupied_cells() == []
    assert state.occupied_cells(PlayerColor.BLUE) == []

    matrix = state.board_matrix()
    assert len(matrix) == 20
    assert all(len(row) == 20 for row in matrix)
    assert all(cell is None for row in matrix for cell in row)

    counts = dict(state.board_counts())
    assert counts[PlayerColor.BLUE] == 0
    assert counts[PlayerColor.YELLOW] == 0
    assert counts[PlayerColor.RED] == 0
    assert counts[PlayerColor.GREEN] == 0


def test_game_state_exposes_board_after_place_move():
    engine = BlocusEngine()
    state = engine.initialize_game(make_config())

    result = engine.apply(state, make_place_command(state))
    next_state = result.next_state

    assert next_state.board_is_empty is False
    assert next_state.cell(0, 0) == PlayerColor.BLUE
    assert next_state.cell(0, 1) is None

    blue_cells = next_state.occupied_cells(PlayerColor.BLUE)
    assert len(blue_cells) == 1
    assert blue_cells[0].row == 0
    assert blue_cells[0].col == 0
    assert blue_cells[0].color == PlayerColor.BLUE

    all_cells = next_state.occupied_cells()
    assert all_cells == blue_cells

    counts = dict(next_state.board_counts())
    assert counts[PlayerColor.BLUE] == 1
    assert counts[PlayerColor.YELLOW] == 0


def test_cell_rejects_invalid_board_coordinates():
    state = make_state()

    with pytest.raises(InputError):
        state.cell(20, 0)

    with pytest.raises(InputError):
        state.cell(0, 20)


def test_engine_exposes_piece_repository_and_orientations():
    engine = BlocusEngine()

    pieces = engine.pieces()
    assert len(pieces) == 21

    first = engine.piece(0)
    assert first.id == 0
    assert first.name
    assert first.square_count >= 1
    assert first.orientation_count >= 1
    assert len(first.orientations) == first.orientation_count

    orientation = first.orientation(0)
    assert orientation.id == 0
    assert orientation.width >= 1
    assert orientation.height >= 1
    assert orientation.square_count == first.square_count
    assert len(orientation.cells) == first.square_count

    absolute_cells = orientation.cells_at(0, 0)
    assert absolute_cells == orientation.cells


def test_piece_lookup_rejects_invalid_ids():
    engine = BlocusEngine()
    piece = engine.piece(0)

    with pytest.raises(InputError):
        engine.piece(255)

    with pytest.raises(InputError):
        piece.orientation(255)


def test_orientation_cells_at_rejects_out_of_bounds_anchor():
    engine = BlocusEngine()
    orientation = engine.piece(0).orientation(0)

    with pytest.raises(InputError):
        orientation.cells_at(20, 0)

    with pytest.raises(InputError):
        orientation.cells_at(0, 20)


def test_game_state_exposes_inventory_views():
    state = make_state()

    assert state.used_piece_ids(PlayerColor.BLUE) == []
    assert state.available_piece_ids(PlayerColor.BLUE) == list(range(21))

    used_pieces = state.used_pieces(PlayerColor.BLUE)
    available_pieces = state.available_pieces(PlayerColor.BLUE)

    assert used_pieces == []
    assert len(available_pieces) == 21
    assert [piece.id for piece in available_pieces] == list(range(21))

    summary = state.inventory_summary(PlayerColor.BLUE)
    assert summary.color == PlayerColor.BLUE
    assert summary.used_piece_ids == []
    assert summary.available_piece_ids == list(range(21))
    assert summary.used_count == 0
    assert summary.available_count == 21
    assert summary.used_square_count == 0
    assert summary.remaining_square_count > 0
    assert summary.is_complete is False
    assert state.remaining_square_count(PlayerColor.BLUE) == summary.remaining_square_count


def test_inventory_views_update_after_move():
    engine = BlocusEngine()
    state = engine.initialize_game(make_config())

    result = engine.apply(state, make_place_command(state))
    next_state = result.next_state

    assert next_state.used_piece_ids(PlayerColor.BLUE) == [0]
    assert 0 not in next_state.available_piece_ids(PlayerColor.BLUE)

    used_pieces = next_state.used_pieces(PlayerColor.BLUE)
    assert [piece.id for piece in used_pieces] == [0]

    summary = next_state.inventory_summary(PlayerColor.BLUE)
    assert summary.used_piece_ids == [0]
    assert summary.used_count == 1
    assert summary.available_count == 20
    assert summary.used_square_count == 1
    assert summary.remaining_square_count == state.remaining_square_count(PlayerColor.BLUE) - 1


def test_engine_exposes_valid_moves_and_piece_filtered_valid_moves():
    engine = BlocusEngine()
    state = engine.initialize_game(make_config())

    moves = engine.get_valid_moves(state, BLUE_PLAYER, PlayerColor.BLUE)
    piece_moves = engine.get_valid_moves_for_piece(
        state,
        BLUE_PLAYER,
        PlayerColor.BLUE,
        0,
    )

    assert moves
    assert piece_moves
    assert all(move.piece_id == 0 for move in piece_moves)
    assert len(piece_moves) <= len(moves)

    assert engine.has_any_valid_move(state, BLUE_PLAYER, PlayerColor.BLUE) is True
    assert engine.has_any_valid_move_for_piece(
        state,
        BLUE_PLAYER,
        PlayerColor.BLUE,
        0,
    ) is True


def test_valid_moves_are_empty_for_wrong_player_and_finished_state():
    engine = BlocusEngine()
    state = engine.initialize_game(make_config())

    assert engine.get_valid_moves(state, YELLOW_GREEN_PLAYER, PlayerColor.BLUE) == []
    assert engine.has_any_valid_move(state, YELLOW_GREEN_PLAYER, PlayerColor.BLUE) is False

    data = json.loads(state.to_json())
    data["status"] = "finished"
    finished = GameState.from_json(json.dumps(data))

    assert engine.get_valid_moves(finished, BLUE_PLAYER, PlayerColor.BLUE) == []
    assert engine.has_any_valid_move(finished, BLUE_PLAYER, PlayerColor.BLUE) is False


def test_json_parser_rejects_invalid_status_instead_of_defaulting_to_in_progress():
    state = make_state()
    data = json.loads(state.to_json())
    data["status"] = "bogus"

    with pytest.raises(InputError):
        GameState.from_json(json.dumps(data))


def test_from_json_recomputes_hash_and_ignores_input_hash():
    state = make_state()
    data = json.loads(state.to_json())

    original_hash = data["hash"]
    data["hash"] = original_hash + 123456789

    restored = GameState.from_json(json.dumps(data))

    assert restored.hash == original_hash
    assert restored.hash != data["hash"]


def test_from_json_rejects_invalid_board_padding_bits():
    state = make_state()
    data = json.loads(state.to_json())

    lanes = data["board"]["blue"]["lanes"]
    lanes[0] = str(1 << 20)

    with pytest.raises(EngineError):
        GameState.from_json(json.dumps(data))


def test_from_json_rejects_overlapping_color_masks():
    state = make_state()
    data = json.loads(state.to_json())

    data["board"]["blue"]["lanes"][0] = "1"
    data["board"]["yellow"]["lanes"][0] = "1"

    with pytest.raises(EngineError):
        GameState.from_json(json.dumps(data))