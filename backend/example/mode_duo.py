from __future__ import annotations

import blocus_engine as be

"""
Blokus Duo example.

Run from backend/ after building the Python extension:

    uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
    uv run python example/mode_duo.py
"""

GAME_ID = "00000000-0000-0000-0000-000000001203"
BLACK_PLAYER = "00000000-0000-0000-0000-000000000001"
WHITE_PLAYER = "00000000-0000-0000-0000-000000000002"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def place_i1(
    *,
    command_id: int,
    player_id: str,
    color: be.PlayerColor,
    row: int,
    col: int,
) -> be.PlaceCommand:
    return be.PlaceCommand(
        command_id=uuid(command_id),
        game_id=GAME_ID,
        player_id=player_id,
        color=color,
        piece_id=0,
        orientation_id=0,
        row=row,
        col=col,
    )


def main() -> None:
    engine = be.BlocusEngine()
    config = be.GameConfig.duo(
        game_id=GAME_ID,
        black_player=BLACK_PLAYER,
        white_player=WHITE_PLAYER,
        first_color=be.PlayerColor.BLACK,
    )
    state = engine.initialize_game(config)

    print("Blokus Duo mode")
    print(f"  mode:            {state.mode.value}")
    print(f"  board_size:      {state.board_size}x{state.board_size}")
    print(f"  scoring:         {state.scoring.value}")
    print(f"  turn_order:      {[color.value for color in state.turn_order]}")
    print("  active colors:   black, white")
    print("  starting points: (4, 4) and (9, 9)")
    print(f"  first turn:      {state.current_color.value}")

    black_moves = engine.get_valid_moves(state, BLACK_PLAYER, be.PlayerColor.BLACK)
    black_i1_starts = sorted((move.row, move.col) for move in black_moves if move.piece_id == 0)
    print(f"\nBlack legal opening moves: {len(black_moves)}")
    print(f"Black I1 can choose either Duo start: {black_i1_starts}")

    result = engine.apply(
        state,
        place_i1(
            command_id=1,
            player_id=BLACK_PLAYER,
            color=be.PlayerColor.BLACK,
            row=4,
            col=4,
        ),
    )
    state = result.next_state

    print("\nApplied Black I1 at (4, 4)")
    print(f"  response:        {result.response.kind.value}")
    print(f"  next color:      {state.current_color.value}")
    print(f"  board matrix:    {len(state.board_matrix())} rows")

    white_moves = engine.get_valid_moves(state, WHITE_PLAYER, be.PlayerColor.WHITE)
    white_i1_starts = sorted((move.row, move.col) for move in white_moves if move.piece_id == 0)
    print(f"\nWhite legal opening moves: {len(white_moves)}")
    print(f"White I1 must use the remaining start: {white_i1_starts}")

    try:
        engine.apply(
            state,
            place_i1(
                command_id=2,
                player_id=WHITE_PLAYER,
                color=be.PlayerColor.WHITE,
                row=0,
                col=0,
            ),
        )
    except be.RuleViolationError as error:
        print("\nRejected White opening away from the remaining Duo start:")
        print(f"  {error}")

    result = engine.apply(
        state,
        place_i1(
            command_id=3,
            player_id=WHITE_PLAYER,
            color=be.PlayerColor.WHITE,
            row=9,
            col=9,
        ),
    )
    state = result.next_state

    print("\nApplied White I1 at (9, 9)")
    print(f"  next color:      {state.current_color.value}")
    print(f"  counts:          {dict(state.board_counts())}")
    print("  scoring note:    Duo always uses advanced scoring.")

    try:
        engine.score_game(state, be.ScoringMode.BASIC)
    except be.InputError as error:
        print("\nRejected Basic scoring for Duo:")
        print(f"  {error}")


if __name__ == "__main__":
    main()
