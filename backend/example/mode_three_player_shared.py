from __future__ import annotations

import blocus_engine as be

"""
Classic three-player Blokus example with a shared color.

Run from backend/ after building the Python extension:

    uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
    uv run python example/mode_three_player_shared.py
"""

GAME_ID = "00000000-0000-0000-0000-000000001202"
PLAYER_ONE = "00000000-0000-0000-0000-000000000001"
PLAYER_TWO = "00000000-0000-0000-0000-000000000002"
PLAYER_THREE = "00000000-0000-0000-0000-000000000003"


def uuid(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def official_turn_order() -> list[be.PlayerColor]:
    return [
        be.PlayerColor.BLUE,
        be.PlayerColor.YELLOW,
        be.PlayerColor.RED,
        be.PlayerColor.GREEN,
    ]


def place_i1(
    *,
    command_id: int,
    game_id: str,
    player_id: str,
    color: be.PlayerColor,
    row: int,
    col: int,
) -> be.PlaceCommand:
    return be.PlaceCommand(
        command_id=uuid(command_id),
        game_id=game_id,
        player_id=player_id,
        color=color,
        piece_id=0,
        orientation_id=0,
        row=row,
        col=col,
    )


def main() -> None:
    engine = be.BlocusEngine()
    shared_green = be.SharedColorTurn(
        be.PlayerColor.GREEN,
        [PLAYER_ONE, PLAYER_TWO, PLAYER_THREE],
    )
    slots = be.PlayerSlots.three_player(
        [
            (be.PlayerColor.BLUE, PLAYER_ONE),
            (be.PlayerColor.YELLOW, PLAYER_TWO),
            (be.PlayerColor.RED, PLAYER_THREE),
        ],
        shared_green,
    )
    config = be.GameConfig(
        GAME_ID,
        be.GameMode.THREE_PLAYER,
        be.ScoringMode.ADVANCED,
        official_turn_order(),
        slots,
    )
    state = engine.initialize_game(config)

    print("Classic three-player mode")
    print(f"  mode:           {state.mode.value}")
    print(f"  board_size:     {state.board_size}x{state.board_size}")
    print(f"  scoring:        {state.scoring.value}")
    print(f"  turn_order:     {[color.value for color in state.turn_order]}")
    print("  owned colors:   blue, yellow, red")
    print(f"  shared color:   {shared_green.color.value}")
    print(f"  shared players: {shared_green.players}")

    scripted_openings = [
        (PLAYER_ONE, be.PlayerColor.BLUE, 0, 0),
        (PLAYER_TWO, be.PlayerColor.YELLOW, 0, 19),
        (PLAYER_THREE, be.PlayerColor.RED, 19, 19),
    ]

    for command_number, (player_id, color, row, col) in enumerate(scripted_openings, start=1):
        moves = engine.get_valid_moves(state, player_id, color)
        print(f"\n{color.value.title()} opening legal moves: {len(moves)}")

        result = engine.apply(
            state,
            place_i1(
                command_id=command_number,
                game_id=GAME_ID,
                player_id=player_id,
                color=color,
                row=row,
                col=col,
            ),
        )
        state = result.next_state
        print(f"  placed I1 at ({row}, {col})")
        print(f"  next color: {state.current_color.value}")

    green_moves = engine.get_valid_moves(state, PLAYER_ONE, be.PlayerColor.GREEN)
    print(f"\nShared Green legal opening moves for first shared controller: {len(green_moves)}")
    print("Green still uses the classic bottom-left corner, but controller ownership rotates.")

    result = engine.apply(
        state,
        place_i1(
            command_id=4,
            game_id=GAME_ID,
            player_id=PLAYER_ONE,
            color=be.PlayerColor.GREEN,
            row=19,
            col=0,
        ),
    )
    state = result.next_state

    print("\nApplied shared Green I1 at (19, 0)")
    print(f"  response:        {result.response.kind.value}")
    print(f"  next color:      {state.current_color.value}")
    print(f"  occupied cells:  {state.board.occupied_count}")
    print("  scoring note:    the shared color is ignored by three-player final scoring.")


if __name__ == "__main__":
    main()
