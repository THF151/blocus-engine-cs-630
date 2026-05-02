from __future__ import annotations

import blocus_engine as be

"""
Minimal Blocus engine usage example.

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


def main() -> None:
    print(f"Engine linked: {be.engine_health()}")

    engine = be.BlocusEngine()

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

    yellow_moves = engine.get_valid_moves(
        state,
        PLAYER_TWO,
        be.PlayerColor.YELLOW,
    )
    print_moves("Yellow legal opening moves", yellow_moves)

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

    yellow_opening_move = be.PlaceCommand(
        command_id=uuid(3),
        game_id=GAME_ID,
        player_id=PLAYER_TWO,
        color=be.PlayerColor.YELLOW,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=19,  # Yellow's starting corner
    )

    result = engine.apply(state, yellow_opening_move)

    print("\nApplied Yellow opening move")
    print_events(result)

    state = result.next_state
    print_state("State after Yellow opening move", state)

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
