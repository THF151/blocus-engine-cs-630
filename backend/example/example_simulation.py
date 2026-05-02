from __future__ import annotations

"""
Complete greedy Blocus game simulation with:
  - final ANSI board visualization
  - runtime monitoring
  - Python memory monitoring

Run from backend/ after building the Rust Python extension:

    uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
    uv run python example/example_simulation.py
"""

import time
import tracemalloc
from dataclasses import dataclass

import blocus_engine as be


GAME_ID = "00000000-0000-0000-0000-000000000700"
PLAYER_ONE = "00000000-0000-0000-0000-000000000001"
PLAYER_TWO = "00000000-0000-0000-0000-000000000002"

BOARD_SIZE = 20

RESET = "\033[0m"
BOLD = "\033[1m"

ANSI_BY_COLOR = {
    "blue": "\033[94m",
    "yellow": "\033[93m",
    "red": "\033[91m",
    "green": "\033[92m",
}

SYMBOL_BY_COLOR = {
    "blue": "B",
    "yellow": "Y",
    "red": "R",
    "green": "G",
}

PIECE_SHAPES: dict[int, list[tuple[int, int]]] = {
    0: [(0, 0)],
    1: [(0, 0), (0, 1)],
    2: [(0, 0), (0, 1), (0, 2)],
    3: [(0, 0), (1, 0), (1, 1)],
    4: [(0, 0), (0, 1), (0, 2), (0, 3)],
    5: [(0, 0), (0, 1), (1, 0), (1, 1)],
    6: [(0, 0), (0, 1), (0, 2), (1, 1)],
    7: [(0, 0), (1, 0), (2, 0), (2, 1)],
    8: [(0, 0), (0, 1), (1, 1), (1, 2)],
    9: [(0, 1), (1, 0), (1, 1), (1, 2), (2, 2)],
    10: [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
    11: [(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)],
    12: [(0, 1), (1, 1), (2, 0), (2, 1), (3, 0)],
    13: [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0)],
    14: [(0, 0), (0, 1), (0, 2), (1, 1), (2, 1)],
    15: [(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)],
    16: [(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)],
    17: [(0, 0), (1, 0), (1, 1), (2, 1), (2, 2)],
    18: [(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
    19: [(0, 0), (1, 0), (2, 0), (3, 0), (2, 1)],
    20: [(0, 0), (0, 1), (1, 1), (2, 1), (2, 2)],
}


@dataclass
class PerfStats:
    total_turns: int = 0
    move_count: int = 0
    pass_count: int = 0
    generated_move_count: int = 0

    total_runtime_seconds: float = 0.0
    total_movegen_seconds: float = 0.0
    total_apply_seconds: float = 0.0
    total_visualization_seconds: float = 0.0

    slowest_turn_seconds: float = 0.0
    slowest_turn_number: int = 0

    peak_legal_moves: int = 0
    peak_legal_moves_turn: int = 0

    python_memory_current_bytes: int = 0
    python_memory_peak_bytes: int = 0


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def format_seconds(seconds: float) -> str:
    return f"{seconds * 1_000:.3f} ms"


def format_bytes(value: int) -> str:
    units = ["B", "KiB", "MiB", "GiB"]
    size = float(value)

    for unit in units:
        if size < 1024 or unit == units[-1]:
            return f"{size:.2f} {unit}"
        size /= 1024

    raise RuntimeError("unreachable")


def controller_for_color(color: be.PlayerColor) -> str:
    if color in (be.PlayerColor.BLUE, be.PlayerColor.RED):
        return PLAYER_ONE

    return PLAYER_TWO


def normalize(cells: list[tuple[int, int]]) -> list[tuple[int, int]]:
    min_row = min(row for row, _ in cells)
    min_col = min(col for _, col in cells)
    return sorted((row - min_row, col - min_col) for row, col in cells)


def transform(
    cells: list[tuple[int, int]],
    rotation: int,
    flip_horizontal: bool,
) -> list[tuple[int, int]]:
    normalized = normalize(cells)
    width = max(col for _, col in normalized) + 1
    height = max(row for row, _ in normalized) + 1

    transformed: list[tuple[int, int]] = []

    for row, col in normalized:
        if flip_horizontal:
            col = width - 1 - col

        if rotation == 0:
            next_row, next_col = row, col
        elif rotation == 90:
            next_row, next_col = col, height - 1 - row
        elif rotation == 180:
            next_row, next_col = height - 1 - row, width - 1 - col
        elif rotation == 270:
            next_row, next_col = width - 1 - col, row
        else:
            raise ValueError(f"unsupported rotation: {rotation}")

        transformed.append((next_row, next_col))

    return normalize(transformed)


def unique_orientations(piece_id: int) -> list[list[tuple[int, int]]]:
    base = PIECE_SHAPES[piece_id]
    orientations: list[list[tuple[int, int]]] = []

    for flip_horizontal in (False, True):
        for rotation in (0, 90, 180, 270):
            orientation = transform(base, rotation, flip_horizontal)

            if orientation not in orientations:
                orientations.append(orientation)

    return orientations


def piece_cells(
    piece_id: int,
    orientation_id: int,
    anchor_row: int,
    anchor_col: int,
) -> list[tuple[int, int]]:
    orientations = unique_orientations(piece_id)

    if orientation_id >= len(orientations):
        raise ValueError(
            f"piece {piece_id} has no orientation {orientation_id}; "
            f"available orientations: {len(orientations)}"
        )

    return [
        (anchor_row + local_row, anchor_col + local_col)
        for local_row, local_col in orientations[orientation_id]
    ]


def empty_visual_board() -> list[list[str | None]]:
    return [[None for _ in range(BOARD_SIZE)] for _ in range(BOARD_SIZE)]


def record_move_on_visual_board(
    board: list[list[str | None]],
    color: be.PlayerColor,
    move: be.LegalMove,
) -> None:
    color_value = color.value

    for row, col in piece_cells(move.piece_id, move.orientation_id, move.row, move.col):
        if not (0 <= row < BOARD_SIZE and 0 <= col < BOARD_SIZE):
            raise RuntimeError(f"visualization bug: cell ({row}, {col}) is outside the board")

        if board[row][col] is not None:
            raise RuntimeError(
                f"visualization bug: cell ({row}, {col}) is already occupied by {board[row][col]}"
            )

        board[row][col] = color_value


def colored_cell(color_value: str | None) -> str:
    if color_value is None:
        return " ·"

    ansi = ANSI_BY_COLOR[color_value]
    symbol = SYMBOL_BY_COLOR[color_value]
    return f" {ansi}{BOLD}{symbol}{RESET}"


def print_final_board(board: list[list[str | None]]) -> None:
    print("\nFinal board")
    print("   " + " ".join(f"{col:2d}" for col in range(BOARD_SIZE)))

    for row_index, row in enumerate(board):
        rendered_cells = "".join(colored_cell(color_value) for color_value in row)
        print(f"{row_index:2d} {rendered_cells}")

    print("\nLegend")
    print(f"  {ANSI_BY_COLOR['blue']}{BOLD}B{RESET} = Blue")
    print(f"  {ANSI_BY_COLOR['yellow']}{BOLD}Y{RESET} = Yellow")
    print(f"  {ANSI_BY_COLOR['red']}{BOLD}R{RESET} = Red")
    print(f"  {ANSI_BY_COLOR['green']}{BOLD}G{RESET} = Green")
    print("  · = Empty")


def print_state_summary(state: be.GameState) -> None:
    print(
        f"version={state.version}, "
        f"status={state.status.value}, "
        f"current_color={state.current_color.value}, "
        f"board_is_empty={state.board_is_empty}"
    )


def print_result_events(result: be.GameResult) -> None:
    event_names = [event.kind.value for event in result.events]
    print(
        f"  response={result.response.kind.value!r}, "
        f"message={result.response.message!r}, "
        f"events={event_names}"
    )


def choose_move(moves: list[be.LegalMove]) -> be.LegalMove:
    return moves[0]


def print_performance_summary(stats: PerfStats) -> None:
    average_turn_seconds = (
        stats.total_runtime_seconds / stats.total_turns
        if stats.total_turns > 0
        else 0.0
    )

    average_movegen_seconds = (
        stats.total_movegen_seconds / stats.total_turns
        if stats.total_turns > 0
        else 0.0
    )

    average_apply_seconds = (
        stats.total_apply_seconds / stats.total_turns
        if stats.total_turns > 0
        else 0.0
    )

    print("\nPerformance summary")
    print(f"  total runtime:              {format_seconds(stats.total_runtime_seconds)}")
    print(f"  average turn runtime:       {format_seconds(average_turn_seconds)}")
    print(f"  slowest turn runtime:       {format_seconds(stats.slowest_turn_seconds)}")
    print(f"  slowest turn number:        {stats.slowest_turn_number}")
    print()
    print(f"  total move-generation time: {format_seconds(stats.total_movegen_seconds)}")
    print(f"  avg move-generation/turn:   {format_seconds(average_movegen_seconds)}")
    print(f"  total apply time:           {format_seconds(stats.total_apply_seconds)}")
    print(f"  avg apply/turn:             {format_seconds(average_apply_seconds)}")
    print(f"  visualization update time:  {format_seconds(stats.total_visualization_seconds)}")
    print()
    print(f"  total generated moves:      {stats.generated_move_count}")
    print(f"  peak legal moves:           {stats.peak_legal_moves}")
    print(f"  peak legal moves turn:      {stats.peak_legal_moves_turn}")
    print()
    print(f"  Python memory current:      {format_bytes(stats.python_memory_current_bytes)}")
    print(f"  Python memory peak:         {format_bytes(stats.python_memory_peak_bytes)}")


def main() -> None:
    tracemalloc.start()

    total_start = time.perf_counter()

    print(f"Engine linked: {be.engine_health()}")

    engine = be.BlocusEngine()
    visual_board = empty_visual_board()
    stats = PerfStats()

    config = be.GameConfig.two_player(
        game_id=GAME_ID,
        blue_red_player=PLAYER_ONE,
        yellow_green_player=PLAYER_TWO,
        scoring=be.ScoringMode.BASIC,
    )

    state = engine.initialize_game(config)

    print("\nInitial state")
    print_state_summary(state)

    command_number = 1

    while state.status == be.GameStatus.IN_PROGRESS:
        turn_start = time.perf_counter()

        color = state.current_color
        player_id = controller_for_color(color)

        movegen_start = time.perf_counter()
        moves = engine.get_valid_moves(state, player_id, color)
        movegen_seconds = time.perf_counter() - movegen_start

        stats.total_movegen_seconds += movegen_seconds
        stats.generated_move_count += len(moves)

        if len(moves) > stats.peak_legal_moves:
            stats.peak_legal_moves = len(moves)
            stats.peak_legal_moves_turn = command_number

        print(
            f"\nTurn {command_number}: "
            f"player={player_id}, color={color.value}, legal_moves={len(moves)}, "
            f"movegen={format_seconds(movegen_seconds)}"
        )

        if moves:
            move = choose_move(moves)

            print(
                "  applying move: "
                f"piece={move.piece_id}, "
                f"orientation={move.orientation_id}, "
                f"row={move.row}, "
                f"col={move.col}, "
                f"board_index={move.board_index}, "
                f"squares={move.score_delta}"
            )

            command = be.PlaceCommand(
                command_id=uuid(command_number),
                game_id=GAME_ID,
                player_id=player_id,
                color=color,
                piece_id=move.piece_id,
                orientation_id=move.orientation_id,
                row=move.row,
                col=move.col,
            )

            apply_start = time.perf_counter()
            result = engine.apply(state, command)
            apply_seconds = time.perf_counter() - apply_start

            visualization_start = time.perf_counter()
            record_move_on_visual_board(visual_board, color, move)
            visualization_seconds = time.perf_counter() - visualization_start

            stats.total_apply_seconds += apply_seconds
            stats.total_visualization_seconds += visualization_seconds
            stats.move_count += 1

            print(f"  apply={format_seconds(apply_seconds)}")
        else:
            print("  no legal move; passing")

            command = be.PassCommand(
                command_id=uuid(command_number),
                game_id=GAME_ID,
                player_id=player_id,
                color=color,
            )

            apply_start = time.perf_counter()
            result = engine.apply(state, command)
            apply_seconds = time.perf_counter() - apply_start

            stats.total_apply_seconds += apply_seconds
            stats.pass_count += 1

            print(f"  pass_apply={format_seconds(apply_seconds)}")

        print_result_events(result)

        state = result.next_state
        print_state_summary(state)

        turn_seconds = time.perf_counter() - turn_start
        stats.total_turns += 1

        if turn_seconds > stats.slowest_turn_seconds:
            stats.slowest_turn_seconds = turn_seconds
            stats.slowest_turn_number = command_number

        print(f"  turn_runtime={format_seconds(turn_seconds)}")

        command_number += 1

    stats.total_runtime_seconds = time.perf_counter() - total_start
    current_memory, peak_memory = tracemalloc.get_traced_memory()
    stats.python_memory_current_bytes = current_memory
    stats.python_memory_peak_bytes = peak_memory

    tracemalloc.stop()

    print("\nGame finished")
    print(f"  total applied moves: {stats.move_count}")
    print(f"  total passes:        {stats.pass_count}")
    print(f"  final version:       {state.version}")

    print_final_board(visual_board)

    basic_scoreboard = engine.score_game(state, be.ScoringMode.BASIC)

    print(f"\nScoreboard ({basic_scoreboard.scoring.value})")
    for entry in basic_scoreboard.entries:
        print(f"  player={entry.player_id}: score={entry.score}")

    advanced_scoreboard = engine.score_game(state, be.ScoringMode.ADVANCED)

    print(f"\nScoreboard ({advanced_scoreboard.scoring.value})")
    for entry in advanced_scoreboard.entries:
        print(f"  player={entry.player_id}: score={entry.score}")

    print_performance_summary(stats)


if __name__ == "__main__":
    main()