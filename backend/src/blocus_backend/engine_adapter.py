from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class ApplyResult:
    next_state: Any
    event_type: str
    response: str


class EngineUnavailableError(RuntimeError):
    pass


class ClassicEngineAdapter:
    def __init__(self) -> None:
        self._engine_module: Any | None = None
        self._engine: Any | None = None

    def _module(self) -> Any:
        if self._engine_module is None:
            try:
                import blocus_engine as engine_module
            except ModuleNotFoundError as error:
                raise EngineUnavailableError("blocus_engine binding is not installed") from error

            self._engine_module = engine_module

        return self._engine_module

    def _client(self) -> Any:
        if self._engine is None:
            self._engine = self._module().BlocusEngine()
        return self._engine

    def engine_health(self) -> bool:
        try:
            return bool(self._module().engine_health())
        except EngineUnavailableError:
            return False

    def serialize_state(self, state: Any) -> str:
        return str(state.to_json())

    def deserialize_state(self, text: str) -> Any:
        return self._module().GameState.from_json(text)

    def create_game(self, payload: dict[str, Any]) -> Any:
        be = self._module()
        mode = payload["mode"]
        scoring = _scoring_mode(be, payload["scoring"])

        if mode == "two_player":
            players = payload["players"]
            config = be.GameConfig.two_player(
                game_id=payload["game_id"],
                blue_red_player=players["blue_red"],
                yellow_green_player=players["yellow_green"],
                scoring=scoring,
            )
        elif mode == "three_player":
            players = payload["players"]
            shared_green = be.SharedColorTurn(
                be.PlayerColor.GREEN,
                players["shared_green"],
            )
            slots = be.PlayerSlots.three_player(
                [
                    (be.PlayerColor.BLUE, players["blue"]),
                    (be.PlayerColor.YELLOW, players["yellow"]),
                    (be.PlayerColor.RED, players["red"]),
                ],
                shared_green,
            )
            config = be.GameConfig(
                payload["game_id"],
                be.GameMode.THREE_PLAYER,
                scoring,
                [_color(be, color) for color in payload["turn_order"]],
                slots,
            )
        elif mode == "four_player":
            players = payload["players"]
            config = be.GameConfig.four_player(
                game_id=payload["game_id"],
                assignments=[
                    (_color(be, color), player_id) for color, player_id in players.items()
                ],
                scoring=scoring,
                turn_order=[_color(be, color) for color in payload["turn_order"]],
            )
        elif mode == "duo":
            players = payload["players"]
            turn_order = payload["turn_order"]
            config = be.GameConfig.duo(
                game_id=payload["game_id"],
                black_player=players["black"],
                white_player=players["white"],
                scoring=scoring,
                first_color=_color(be, turn_order[0]),
            )
        else:
            raise ValueError(f"Unsupported mode: {mode}")

        return self._client().initialize_game(config)

    def state_view(self, state: Any) -> dict[str, Any]:
        view = {
            "game_id": state.game_id,
            "mode": state.mode.value,
            "scoring": state.scoring.value,
            "status": state.status.value,
            "version": state.version,
            "hash": state.hash,
            "board_size": state.board_size,
            "board_is_empty": state.board_is_empty,
            "current_color": state.current_color.value,
            "turn_order": [color.value for color in state.turn_order],
            "occupied_count": state.board.occupied_count,
            "board_counts": [
                {"color": color.value, "count": count} for color, count in state.board_counts()
            ],
            # Full cell-level snapshot so the frontend can render the board
            # without tracking every move incrementally.
            "board_cells": [
                {"row": cell.row, "col": cell.col, "color": cell.color.value}
                for cell in state.board.occupied
            ],
        }
        shared_color_turn_index = _shared_color_turn_index(state)
        if shared_color_turn_index is not None:
            view["shared_color_turn_index"] = shared_color_turn_index
        return view

    def legal_moves(self, state: Any, player_id: str, color: str) -> list[dict[str, int]]:
        moves = self._client().get_valid_moves(state, player_id, _color(self._module(), color))
        return [_legal_move_view(move) for move in moves]

    def place_move(self, state: Any, payload: dict[str, Any]) -> ApplyResult:
        be = self._module()
        command = be.PlaceCommand(
            command_id=payload["command_id"],
            game_id=payload["game_id"],
            player_id=payload["player_id"],
            color=_color(be, payload["color"]),
            piece_id=payload["piece_id"],
            orientation_id=payload["orientation_id"],
            row=payload["row"],
            col=payload["col"],
        )
        result = self._client().apply(state, command)
        return ApplyResult(
            next_state=result.next_state,
            event_type="move_applied",
            response=result.response.message,
        )

    def pass_move(self, state: Any, payload: dict[str, Any]) -> ApplyResult:
        be = self._module()
        command = be.PassCommand(
            command_id=payload["command_id"],
            game_id=payload["game_id"],
            player_id=payload["player_id"],
            color=_color(be, payload["color"]),
        )
        result = self._client().apply(state, command)
        return ApplyResult(
            next_state=result.next_state,
            event_type="pass_applied",
            response=result.response.message,
        )

    def score_game(self, state: Any) -> dict[str, Any]:
        scoreboard = self._client().score_game(state, state.scoring)
        return {
            "scoring": scoreboard.scoring.value,
            "entries": [
                {"player_id": entry.player_id, "score": entry.score} for entry in scoreboard.entries
            ],
        }


def _color(be: Any, value: str) -> Any:
    colors = {
        "blue": be.PlayerColor.BLUE,
        "yellow": be.PlayerColor.YELLOW,
        "red": be.PlayerColor.RED,
        "green": be.PlayerColor.GREEN,
        "black": be.PlayerColor.BLACK,
        "white": be.PlayerColor.WHITE,
    }
    return colors[value]


def _scoring_mode(be: Any, value: str) -> Any:
    if value == "advanced":
        return be.ScoringMode.ADVANCED
    return be.ScoringMode.BASIC


def _shared_color_turn_index(state: Any) -> int | None:
    try:
        data = json.loads(str(state.to_json()))
    except TypeError, ValueError:
        return None

    if not isinstance(data, dict):
        return None
    turn = data.get("turn")
    if not isinstance(turn, dict):
        return None
    index = turn.get("shared_color_turn_index")
    if not isinstance(index, int):
        return None
    return index


def _legal_move_view(move: Any) -> dict[str, int]:
    return {
        "piece_id": move.piece_id,
        "orientation_id": move.orientation_id,
        "row": move.row,
        "col": move.col,
        "board_index": move.board_index,
        "score_delta": move.score_delta,
    }
