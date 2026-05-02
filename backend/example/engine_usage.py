from __future__ import annotations

"""
Minimal Blocus engine usage example.

Run from backend/ after building the Rust Python extension:

    uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
    uv run python example/engine_usage.py
"""

import blocus_engine as be


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

    opening_move = be.PlaceCommand(
        command_id=uuid(1),
        game_id=GAME_ID,
        player_id=PLAYER_ONE,
        color=be.PlayerColor.BLUE,
        piece_id=0,          # I1, the one-square piece
        orientation_id=0,    # I1 has exactly one orientation
        row=0,
        col=0,               # Blue's starting corner
    )

    result = engine.apply(state, opening_move)

    print("\nApplied opening move")
    print(f"  response: {result.response.kind} - {result.response.message}")
    print("  events:")
    for event in result.events:
        print(f"    - {event.kind} at version {event.version}")

    print_state("Next state", result.next_state)

    illegal_move = be.PlaceCommand(
        command_id=uuid(2),
        game_id=GAME_ID,
        player_id=PLAYER_TWO,
        color=be.PlayerColor.YELLOW,
        piece_id=0,
        orientation_id=0,
        row=0,
        col=18,              # Yellow's first move must cover row 0, col 19
    )

    try:
        engine.apply(result.next_state, illegal_move)
    except be.RuleViolationError as error:
        print("\nRejected illegal move")
        print(f"  {error}")


if __name__ == "__main__":
    main()