from __future__ import annotations

import time
import tracemalloc
from dataclasses import dataclass

import blocus_engine as be

"""
Complete greedy Blocus game simulation with:
  - final ANSI board visualization
  - runtime monitoring
  - Python memory monitoring

Run from backend/ after building the Rust Python extension:

    uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
    uv run python example/example_simulation.py
"""


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


@dataclass
class PerfStats:
    total_turns: int = 0
    move_count: int = 0
    pass_count: int = 0
    generated_move_count: int = 0

    total_runtime_seconds: float = 0.0
    total_movegen_seconds: float = 0.0
    total_apply_seconds: float = 0.0

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


def print_all_pieces(engine: be.BlocusEngine) -> None:
    print("\nCanonical Pieces (Orientation 0):")
    for piece in engine.pieces():
        print(
            f"\n  {piece.id:2d}: {piece.name} "
            f"({piece.square_count} sq, {piece.orientation_count} ori)"
        )
        base = piece.orientations[0]
        grid = [[" " for _ in range(base.width)] for _ in range(base.height)]
        for r, c in base.cells:
            grid[r][c] = "█"
        for row in grid:
            print("      " + "".join(row))


def colored_cell_from_color(color: be.PlayerColor | None) -> str:
    if color is None:
        return " ·"

    color_value = color.value
    ansi = ANSI_BY_COLOR[color_value]
    symbol = SYMBOL_BY_COLOR[color_value]
    return f" {ansi}{BOLD}{symbol}{RESET}"


def print_final_state_board(state: be.GameState) -> None:
    print("\nFinal board")
    print("   " + " ".join(f"{col:2d}" for col in range(BOARD_SIZE)))

    matrix = state.board_matrix()
    for row_index, row in enumerate(matrix):
        rendered_cells = "".join(colored_cell_from_color(color) for color in row)
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
        stats.total_runtime_seconds / stats.total_turns if stats.total_turns > 0 else 0.0
    )

    average_movegen_seconds = (
        stats.total_movegen_seconds / stats.total_turns if stats.total_turns > 0 else 0.0
    )

    average_apply_seconds = (
        stats.total_apply_seconds / stats.total_turns if stats.total_turns > 0 else 0.0
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
    print_all_pieces(engine)

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

            stats.total_apply_seconds += apply_seconds
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

    print_final_state_board(state)

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
