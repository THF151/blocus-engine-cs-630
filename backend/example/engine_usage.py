from __future__ import annotations

import blocus_engine as be

"""
Comprehensive Blocus engine usage example.

Run from backend/ after building the Rust Python extension:

    uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
    uv run python example/engine_usage.py
"""

GAME_ID = "00000000-0000-0000-0000-000000000100"
PLAYER_ONE = "00000000-0000-0000-0000-000000000001"
PLAYER_TWO = "00000000-0000-0000-0000-000000000002"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def print_state(label: str, state: be.GameState) -> None:
    print(f"\n{label}")
    print(f"  game_id:        {state.game_id}")
    print(f"  mode:           {state.mode.value}")
    print(f"  scoring:        {state.scoring.value}")
    print(f"  status:         {state.status.value}")
    print(f"  version:        {state.version}")
    print(f"  hash:           {state.hash}")
    print(f"  board_is_empty: {state.board_is_empty}")
    print(f"  current_color:  {state.current_color.value}")
    print(f"  turn_order:     {[color.value for color in state.turn_order]}")


def print_events(result: be.GameResult) -> None:
    print(f"  response: {result.response.kind.value} - {result.response.message}")
    print("  events:")
    for event in result.events:
        print(f"    - {event.kind.value} at version {event.version}")


def print_moves(label: str, moves: list[be.LegalMove], limit: int = 5) -> None:
    print(f"\n{label}")
    print(f"  total legal moves: {len(moves)}")

    for move in moves[:limit]:
        print(
            "  "
            f"piece={move.piece_id}, "
            f"orientation={move.orientation_id}, "
            f"row={move.row}, "
            f"col={move.col}, "
            f"board_index={move.board_index}, "
            f"squares={move.score_delta}"
        )


def print_scoreboard(scoreboard: be.ScoreBoard) -> None:
    print(f"\nScoreboard ({scoreboard.scoring.value})")
    for entry in scoreboard.entries:
        print(f"  player={entry.player_id}: score={entry.score}")


def demo_pieces_api(engine: be.BlocusEngine) -> None:
    print("\n--- Pieces API ---")
    pieces = engine.pieces()
    print(f"Engine loaded {len(pieces)} canonical pieces.")

    # Inspect a specific piece (e.g., piece 3, the V3 piece)
    v3 = engine.piece(3)
    print(
        f"Piece {v3.id} is '{v3.name}' with {v3.square_count} squares and {v3.orientation_count} unique orientations.")

    print("Orientations:")
    for ori in v3.orientations:
        print(f"  Orientation {ori.id}: {ori.width}x{ori.height} bounds, cells={ori.cells}")


def demo_inventory_api(state: be.GameState, color: be.PlayerColor) -> None:
    print(f"\n--- Inventory API ({color.value}) ---")
    summary = state.inventory_summary(color)
    print(f"  Used piece IDs:        {summary.used_piece_ids}")
    print(f"  Available piece count: {summary.available_count}")
    print(f"  Remaining squares:     {summary.remaining_square_count}")
    print(f"  Is complete?           {summary.is_complete}")


def demo_board_api(state: be.GameState) -> None:
    print("\n--- Board API ---")
    print(f"  Total occupied cells: {state.board.occupied_count}")

    # Board counts by color
    print("  Cell counts by color:")
    for color, count in state.board_counts():
        if count > 0:
            print(f"    {color.value}: {count}")

    # Inspect specific occupied cells
    blue_cells = state.occupied_cells(be.PlayerColor.BLUE)
    if blue_cells:
        print(f"  Blue occupies cells like: (row={blue_cells[0].row}, col={blue_cells[0].col})")

    # Render a compact slice of the board matrix (top left 10x10 and top right 10x10)
    print("  Board Matrix (Top 5 rows):")
    matrix = state.board_matrix()
    for row in matrix[:5]:
        rendered = "".join(" ." if cell is None else f" {cell.name[0]}" for cell in row)
        print(f"    {rendered}")


def demo_serialization_api(state: be.GameState) -> None:
    print("\n--- Serialization API ---")

    # Export state to JSON
    json_data = state.to_json()
    print(f"  Serialized to JSON string ({len(json_data)} bytes).")
    print(f"  JSON Preview: {json_data[:150]}...")

    # Re-hydrate state from JSON
    restored_state = be.GameState.from_json(json_data)
    print(f"  Restored state version: {restored_state.version}")
    print(f"  Original hash: {state.hash}")
    print(f"  Restored hash: {restored_state.hash}")

    if state.hash == restored_state.hash:
        print("  SUCCESS: Restored state hash exactly matches original hash.")
    else:
        print("  ERROR: Hash mismatch on restored state.")


def main() -> None:
    print(f"Engine linked: {be.engine_health()}")

    engine = be.BlocusEngine()

    # 1. Explore Pieces
    demo_pieces_api(engine)

    # 2. Game Setup
    config = be.GameConfig.two_player(
        game_id=GAME_ID,
        blue_red_player=PLAYER_ONE,
        yellow_green_player=PLAYER_TWO,
        scoring=be.ScoringMode.BASIC,
    )

    state = engine.initialize_game(config)
    print_state("Initial state", state)

    blue_opening_moves = engine.get_valid_moves(
        state,
        PLAYER_ONE,
        be.PlayerColor.BLUE,
    )
    print_moves("Blue legal opening moves", blue_opening_moves)

    print(
        "\nBlue has any valid move:",
        engine.has_any_valid_move(state, PLAYER_ONE, be.PlayerColor.BLUE),
    )

    # 3. Apply Blue Move
    opening_move = be.PlaceCommand(
        command_id=uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_ONE,
        color=be.PlayerColor.BLUE,
        piece_id=0,  # I1, the one-square piece
        orientation_id=0,  # I1 has exactly one orientation
        row=0,
        col=0,  # Blue's starting corner
    )

    result = engine.apply(state, opening_move)

    print("\nApplied opening move")
    print_events(result)

    state = result.next_state
    print_state("State after Blue opening move", state)

    # 4. Explore Inventory Updates
    demo_inventory_api(state, be.PlayerColor.BLUE)

    yellow_moves = engine.get_valid_moves(
        state,
        PLAYER_TWO,
        be.PlayerColor.YELLOW,
    )
    print_moves("Yellow legal opening moves", yellow_moves)

    # 5. Intentional Rule Violations
    illegal_pass = be.PassCommand(
        command_id=uuid(2),
        game_id=GAME_ID,
        player_id=PLAYER_TWO,
        color=be.PlayerColor.YELLOW,
    )

    try:
        engine.apply(state, illegal_pass)
    except be.RuleViolationError as error:
        print("\nRejected pass while Yellow still has a legal move")
        print(f"  {error}")

    # 6. Apply Yellow Move
    yellow_opening_move = be.PlaceCommand(
        command_id=uuid(3),
        game_id=GAME_ID,
        player_id=PLAYER_TWO,
        color=be.PlayerColor.YELLOW,
        piece_id=4,  # I4 piece
        orientation_id=0,
        row=0,
        col=16,  # Valid placement for a length 4 piece starting at 19 and growing left depending on rotation
    )

    # Wait, lets make sure Yellow places exactly at its corner.
    # The piece 0 (I1) is the safest brute-force corner placement if we don't calculate orientation offsets here.
    yellow_safe_move = be.PlaceCommand(
        command_id=uuid(3),
        game_id=GAME_ID,
        player_id=PLAYER_TWO,
        color=be.PlayerColor.YELLOW,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=19,
    )

    result = engine.apply(state, yellow_safe_move)

    print("\nApplied Yellow opening move")
    print_events(result)

    state = result.next_state

    # 7. Explore Board API
    demo_board_api(state)

    # 8. Explore State Serialization API
    demo_serialization_api(state)

    unfinished_score_state = state
    try:
        engine.score_game(unfinished_score_state, be.ScoringMode.BASIC)
    except be.RuleViolationError as error:
        print("\nRejected scoring before the game is finished")
        print(f"  {error}")

    print(
        "\nNote: final scoring requires a finished GameState. "
        "The current public Python API does not expose a fixture builder or mutable "
        "state editor, so this example shows the scoring call and its typed rejection "
        "for an unfinished game."
    )


if __name__ == "__main__":
    main()