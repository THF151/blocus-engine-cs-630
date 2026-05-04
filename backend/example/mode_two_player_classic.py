from __future__ import annotations

import blocus_engine as be

"""
Classic two-player Blokus example.

Run from backend/ after building the Python extension:

    uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
    uv run python example/mode_two_player_classic.py
"""

GAME_ID = "00000000-0000-0000-0000-000000001201"
PLAYER_ONE = "00000000-0000-0000-0000-000000000001"
PLAYER_TWO = "00000000-0000-0000-0000-000000000002"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def print_counts(state: be.GameState) -> None:
    print("Board counts:")
    for color, count in state.board_counts():
        print(f"  {color.value:6s}: {count}")


def main() -> None:
    engine = be.BlocusEngine()
    config = be.GameConfig.two_player(
        game_id=GAME_ID,
        blue_red_player=PLAYER_ONE,
        yellow_green_player=PLAYER_TWO,
        scoring=be.ScoringMode.BASIC,
    )
    state = engine.initialize_game(config)

    print("Classic two-player mode")
    print(f"  mode:        {state.mode.value}")
    print(f"  board_size:  {state.board_size}x{state.board_size}")
    print(f"  scoring:     {state.scoring.value}")
    print(f"  turn_order:  {[color.value for color in state.turn_order]}")
    print("  ownership:   player one controls blue/red; player two controls yellow/green")
    print(f"  first turn:  {state.current_color.value}")

    blue_moves = engine.get_valid_moves(state, PLAYER_ONE, be.PlayerColor.BLUE)
    print(f"\nBlue legal opening moves: {len(blue_moves)}")
    print("Blue must cover the top-left classic corner.")

    result = engine.apply(
        state,
        be.PlaceCommand(
            command_id=uuid(1),
            game_id=GAME_ID,
            player_id=PLAYER_ONE,
            color=be.PlayerColor.BLUE,
            piece_id=0,
            orientation_id=0,
            row=0,
            col=0,
        ),
    )
    state = result.next_state

    print("\nApplied Blue I1 at (0, 0)")
    print(f"  response:    {result.response.kind.value}")
    print(f"  next turn:   {state.current_color.value}")
    print(f"  blue used:   {state.inventory_summary(be.PlayerColor.BLUE).used_piece_ids}")
    print_counts(state)

    yellow_moves = engine.get_valid_moves(state, PLAYER_TWO, be.PlayerColor.YELLOW)
    print(f"\nYellow legal opening moves: {len(yellow_moves)}")
    print("Yellow must cover the top-right classic corner.")

    try:
        engine.apply(
            state,
            be.PassCommand(
                command_id=uuid(2),
                game_id=GAME_ID,
                player_id=PLAYER_TWO,
                color=be.PlayerColor.YELLOW,
            ),
        )
    except be.RuleViolationError as error:
        print("\nPass rejected while Yellow has legal moves:")
        print(f"  {error}")


if __name__ == "__main__":
    main()
