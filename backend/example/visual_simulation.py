"""Visual Blocus simulation CLI.

Connects to a running FastAPI backend over WebSocket, creates a game in the
selected mode, attaches AI players, and renders the game live using Rich:
header with game metadata, turn-order panel, score-and-counts table, the
full board grid, and a rolling move log. Useful for eyeballing that all
the moving parts (CAS, AI loop, broadcasts, seat binding) work end-to-end.

Usage:
    python backend/example/visual_simulation.py
    python backend/example/visual_simulation.py --mode duo
    python backend/example/visual_simulation.py --mode four_player --first-color red
    python backend/example/visual_simulation.py --url ws://localhost:8000/ws --tick 0.25

The script subscribes with a real ``player_id`` to exercise the seat-binding
path (Step 3.5) and attaches an AI to every seat so the game plays itself.
"""

from __future__ import annotations

import argparse
import json
import time
from collections.abc import Iterable
from dataclasses import dataclass, field
from typing import Any
from uuid import uuid4

from rich.align import Align
from rich.console import Console, Group
from rich.layout import Layout
from rich.live import Live
from rich.panel import Panel
from rich.table import Table
from rich.text import Text
from websockets.sync.client import connect as ws_connect

DEFAULT_URL = "ws://localhost:8000/ws"
MAX_LOG_LINES = 14

# Color → (Rich style, glyph for board cell, glyph for legend swatch)
_COLOR_STYLE: dict[str, tuple[str, str]] = {
    "blue": ("bold blue", "■"),
    "yellow": ("bold yellow", "■"),
    "red": ("bold red", "■"),
    "green": ("bold green", "■"),
    "black": ("bold bright_black", "■"),
    "white": ("bold white on grey15", "■"),
}


@dataclass
class GameUI:
    mode: str
    players: dict[str, Any]
    state: dict[str, Any] | None = None
    move_log: list[str] = field(default_factory=list)
    status_line: str = "connecting…"

    def push_log(self, line: str) -> None:
        self.move_log.append(line)
        if len(self.move_log) > MAX_LOG_LINES:
            self.move_log = self.move_log[-MAX_LOG_LINES:]


def main() -> None:
    args = _parse_args()
    players = _players_for_mode(args.mode)
    ai_seats = list(_ai_seats_for_mode(args.mode))
    game_id = str(uuid4())
    observer_player_id = _player_id(1)

    ui = GameUI(mode=args.mode, players=players)
    console = Console()

    with (
        ws_connect(args.url) as websocket,
        Live(
            _render(ui),
            console=console,
            refresh_per_second=8,
            screen=False,
        ) as live,
    ):
        ui.status_line = f"creating {args.mode} game {game_id[:8]}…"
        live.update(_render(ui))

        _send(
            websocket,
            {
                "action": "create_game",
                "payload": {
                    "game_id": game_id,
                    "mode": args.mode,
                    "scoring": _scoring_for_mode(args.mode),
                    "players": players,
                    **({"first_color": args.first_color} if args.first_color else {}),
                },
            },
        )
        _wait_for(websocket, {"game_created"})

        # Subscribe with a real player_id to exercise the seat-binding path
        ui.status_line = f"subscribing as {observer_player_id[-8:]}…"
        live.update(_render(ui))
        _send(
            websocket,
            {
                "action": "subscribe_game",
                "payload": {"game_id": game_id, "player_id": observer_player_id},
            },
        )
        snapshot = _wait_for(websocket, {"state_snapshot"})
        ui.state = snapshot["state"]
        ui.push_log(f"[bold cyan]subscribed[/] as {observer_player_id[-8:]}")
        live.update(_render(ui))

        # Attach AIs after subscribing so the AI loop respects our binding
        # (Binding ≻ AI). Then detach the observer by leaving — for the CLI
        # we don't actually move, we just spectate the AI chain.
        for player_id, color in ai_seats:
            _send(
                websocket,
                {
                    "action": "attach_ai",
                    "payload": {"game_id": game_id, "player_id": player_id, "color": color},
                },
            )
            ui.push_log(f"AI attached: {_swatch(color)} {color} → {player_id[-8:]}")
            live.update(_render(ui))

        # The observer is bound, which prevents the AI from playing for it.
        # Re-subscribe without player_id to drop the binding and let the
        # full AI chain run.
        _send(
            websocket,
            {
                "action": "subscribe_game",
                "payload": {"game_id": game_id},
            },
        )
        _wait_for(websocket, {"state_snapshot"})
        ui.push_log("[dim]observer released seat — AI takes over[/dim]")
        ui.status_line = "AI playing…"
        live.update(_render(ui))

        # Stream events until the game finishes
        while True:
            event = _recv(websocket)
            etype = event.get("type")

            if etype in {"move_applied", "pass_applied", "game_finished"}:
                state = event["state"]
                ui.state = state
                color = state["current_color"]
                color_text = f"[{_COLOR_STYLE.get(color, ('white', ''))[0]}]{color}[/]"
                resp = event.get("response", etype)
                ui.push_log(f"v{state['version']:>3}  {color_text}  {resp}")
                ui.status_line = f"[bold]{etype}[/]  v{state['version']}  next: {color_text}"
                live.update(_render(ui))
                if etype == "game_finished":
                    break
            elif etype == "state_snapshot":
                ui.state = event["state"]
                live.update(_render(ui))
            elif etype == "error":
                ui.push_log(f"[red]error: {event.get('code')} — {event.get('message')}[/red]")
                ui.status_line = f"[red]error: {event.get('code')}[/red]"
                live.update(_render(ui))
                break
            elif etype == "kicked":
                ui.push_log(f"[red]kicked: {event.get('reason')}[/red]")
                break

            time.sleep(args.tick)

        # Final score report
        _send(websocket, {"action": "request_score", "payload": {"game_id": game_id}})
        score = _wait_for(websocket, {"score_report"})
        ui.status_line = "[bold green]game complete[/]"
        ui.push_log("[bold green]final score:[/]")
        for entry in score["score"]["entries"]:
            ui.push_log(f"  {entry['player_id'][-8:]}: {entry['score']}")
        live.update(_render(ui))
        time.sleep(2)


# --- Rendering ----------------------------------------------------------------


def _render(ui: GameUI) -> Layout:
    layout = Layout()
    layout.split_column(
        Layout(_header_panel(ui), name="header", size=3),
        Layout(name="body", ratio=1),
        Layout(_footer_panel(ui), name="footer", size=3),
    )
    layout["body"].split_row(
        Layout(_left_column(ui), name="left", ratio=1),
        Layout(_board_panel(ui), name="board", ratio=2),
    )
    return layout


def _header_panel(ui: GameUI) -> Panel:
    state = ui.state or {}
    title = Text("Blocus", style="bold magenta")
    parts = [
        title,
        Text(f"  mode={state.get('mode', ui.mode)}", style="dim"),
        Text(f"  scoring={state.get('scoring', '?')}", style="dim"),
        Text(f"  v{state.get('version', 0)}", style="dim"),
        Text(f"  status={state.get('status', '?')}", style="dim"),
    ]
    return Panel(
        Align.left(Text.assemble(*parts)),
        title=f"game {state.get('game_id', '?')[:8]}",
        title_align="left",
        border_style="bright_blue",
    )


def _left_column(ui: GameUI) -> Group:
    return Group(_turn_panel(ui), _scores_panel(ui), _log_panel(ui))


def _turn_panel(ui: GameUI) -> Panel:
    state = ui.state or {}
    current = state.get("current_color", "")
    turn_order: list[str] = list(state.get("turn_order", []))

    table = Table(show_header=False, expand=True, box=None, padding=(0, 1))
    table.add_column("marker", width=2)
    table.add_column("color", ratio=1)
    table.add_column("player", ratio=2)

    for color in turn_order:
        marker = "[bold green]▶[/]" if color == current else " "
        style, _ = _COLOR_STYLE.get(color, ("white", ""))
        color_text = f"[{style}]{color}[/]"
        player = _player_for_color(ui.mode, ui.players, color, state)
        table.add_row(marker, color_text, player[-12:])

    return Panel(table, title="turn order", title_align="left", border_style="cyan")


def _scores_panel(ui: GameUI) -> Panel:
    state = ui.state or {}
    counts = state.get("board_counts", [])

    table = Table(show_header=True, expand=True, box=None, header_style="dim")
    table.add_column("color", ratio=1)
    table.add_column("squares", justify="right", ratio=1)

    for entry in counts:
        color = entry["color"]
        style, _ = _COLOR_STYLE.get(color, ("white", ""))
        table.add_row(f"[{style}]{color}[/]", str(entry["count"]))

    table.add_row("", "")
    table.add_row("[bold]total[/]", f"[bold]{state.get('occupied_count', 0)}[/]")

    return Panel(table, title="board counts", title_align="left", border_style="cyan")


def _log_panel(ui: GameUI) -> Panel:
    body = Text("\n").join(Text.from_markup(line) for line in ui.move_log)
    return Panel(body, title="event log", title_align="left", border_style="cyan")


def _board_panel(ui: GameUI) -> Panel:
    state = ui.state
    if state is None or "board" not in state:
        return Panel(Align.center(Text("(no state yet)", style="dim")), border_style="dim")

    board: list[list[str | None]] = state["board"]
    rows = []
    for row in board:
        line = Text()
        for cell in row:
            if cell is None:
                line.append("· ", style="grey30")
            else:
                style, glyph = _COLOR_STYLE.get(cell, ("white", "■"))
                line.append(glyph + " ", style=style)
        rows.append(line)

    body = Text("\n").join(rows)
    size = state.get("board_size", len(board))
    return Panel(
        Align.center(body),
        title=f"board {size}×{size}",
        title_align="left",
        border_style="magenta",
    )


def _footer_panel(ui: GameUI) -> Panel:
    return Panel(
        Align.left(Text.from_markup(ui.status_line)),
        border_style="bright_black",
    )


def _swatch(color: str) -> str:
    style, glyph = _COLOR_STYLE.get(color, ("white", "■"))
    return f"[{style}]{glyph}[/]"


# --- WebSocket plumbing -------------------------------------------------------


def _send(websocket: Any, message: dict[str, Any]) -> None:
    websocket.send(json.dumps(message))


def _recv(websocket: Any) -> dict[str, Any]:
    raw = websocket.recv()
    data = json.loads(raw)
    if not isinstance(data, dict):
        raise RuntimeError(f"unexpected message: {raw!r}")
    return data


def _wait_for(websocket: Any, event_types: set[str]) -> dict[str, Any]:
    while True:
        event = _recv(websocket)
        if event.get("type") == "error":
            raise RuntimeError(event)
        if event.get("type") in event_types:
            return event


# --- Mode-specific config -----------------------------------------------------


def _players_for_mode(mode: str) -> dict[str, Any]:
    if mode == "two_player":
        return {"blue_red": _player_id(1), "yellow_green": _player_id(2)}
    if mode == "three_player":
        return {
            "blue": _player_id(1),
            "yellow": _player_id(2),
            "red": _player_id(3),
            "shared_green": [_player_id(1), _player_id(2), _player_id(3)],
        }
    if mode == "four_player":
        return {
            "blue": _player_id(1),
            "yellow": _player_id(2),
            "red": _player_id(3),
            "green": _player_id(4),
        }
    if mode == "duo":
        return {"black": _player_id(1), "white": _player_id(2)}
    raise ValueError(f"unsupported mode: {mode}")


def _ai_seats_for_mode(mode: str) -> Iterable[tuple[str, str]]:
    if mode == "two_player":
        return [
            (_player_id(1), "blue"),
            (_player_id(2), "yellow"),
            (_player_id(1), "red"),
            (_player_id(2), "green"),
        ]
    if mode == "three_player":
        return [
            (_player_id(1), "blue"),
            (_player_id(2), "yellow"),
            (_player_id(3), "red"),
            (_player_id(1), "green"),
            (_player_id(2), "green"),
            (_player_id(3), "green"),
        ]
    if mode == "four_player":
        return [
            (_player_id(1), "blue"),
            (_player_id(2), "yellow"),
            (_player_id(3), "red"),
            (_player_id(4), "green"),
        ]
    if mode == "duo":
        return [(_player_id(1), "black"), (_player_id(2), "white")]
    raise ValueError(f"unsupported mode: {mode}")


def _scoring_for_mode(mode: str) -> str:
    return "advanced" if mode == "duo" else "basic"


def _player_for_color(mode: str, players: dict[str, Any], color: str, state: dict[str, Any]) -> str:
    if mode == "two_player":
        return players["blue_red"] if color in {"blue", "red"} else players["yellow_green"]
    if mode == "three_player":
        if color == "green":
            shared = players["shared_green"]
            index = int(state.get("shared_color_turn_index", 0))
            return shared[index % len(shared)]
        return players[color]
    if mode == "duo":
        return players["black"] if color == "black" else players["white"]
    return players[color]


def _player_id(value: int) -> str:
    return f"00000000-0000-0000-0000-{value:012d}"


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default=DEFAULT_URL, help="WebSocket URL")
    parser.add_argument(
        "--mode",
        choices=["two_player", "three_player", "four_player", "duo"],
        default="four_player",
    )
    parser.add_argument(
        "--first-color",
        default=None,
        help="Override the starting color (four_player: blue/yellow/red/green; duo: black/white)",
    )
    parser.add_argument(
        "--tick",
        type=float,
        default=0.15,
        help="Seconds to sleep between events for readability (default 0.15)",
    )
    return parser.parse_args()


if __name__ == "__main__":
    main()
