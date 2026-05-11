"""Pydantic models for the websocket protocol.

These define the *structural* contract for each action's payload (required
fields, primitive types, list shape). Domain-level validation that needs a
granular wire error code lives in ``service.py``:

- mode set membership → ``invalid_mode``
- Classic color set membership → ``invalid_classic_color``
- Duo color set membership → ``invalid_duo_color``
- scoring set membership → ``invalid_scoring``

Pydantic ``ValidationError`` is mapped to ``invalid_players`` for failures
under ``players``, ``invalid_scoring`` for failures under ``scoring``, or
``missing_field`` otherwise.
"""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field


class TwoPlayerPlayers(BaseModel):
    blue_red: str = Field(min_length=1)
    yellow_green: str = Field(min_length=1)


class ThreePlayerPlayers(BaseModel):
    blue: str = Field(min_length=1)
    yellow: str = Field(min_length=1)
    red: str = Field(min_length=1)
    shared_green: list[str] = Field(min_length=1)


class FourPlayerPlayers(BaseModel):
    blue: str = Field(min_length=1)
    yellow: str = Field(min_length=1)
    red: str = Field(min_length=1)
    green: str = Field(min_length=1)


class _CreateBase(BaseModel):
    game_id: str | None = None
    scoring: str = "basic"


class TwoPlayerCreate(_CreateBase):
    mode: str
    players: TwoPlayerPlayers


class ThreePlayerCreate(_CreateBase):
    mode: str
    players: ThreePlayerPlayers


class FourPlayerCreate(_CreateBase):
    mode: str
    players: FourPlayerPlayers
    first_color: str = "blue"


class DuoPlayers(BaseModel):
    black: str = Field(min_length=1)
    white: str = Field(min_length=1)


class DuoCreateGameRequest(BaseModel):
    """Duo is advanced-scoring-only by engine constraint; the Literal here
    surfaces a clean ``invalid_scoring`` error if a client sends ``basic``
    instead of letting the engine raise a generic ``invalid_command``."""

    game_id: str | None = None
    mode: str
    players: DuoPlayers
    first_color: str = "black"
    scoring: Literal["advanced"] = "advanced"


class LegalMovesRequest(BaseModel):
    game_id: str = Field(min_length=1)
    player_id: str = Field(min_length=1)
    color: str = Field(min_length=1)


class PlaceMoveRequest(BaseModel):
    game_id: str = Field(min_length=1)
    command_id: str = Field(min_length=1)
    player_id: str = Field(min_length=1)
    color: str = Field(min_length=1)
    piece_id: int
    orientation_id: int
    row: int
    col: int


class PassMoveRequest(BaseModel):
    game_id: str = Field(min_length=1)
    command_id: str = Field(min_length=1)
    player_id: str = Field(min_length=1)
    color: str = Field(min_length=1)


class AttachAiRequest(BaseModel):
    game_id: str = Field(min_length=1)
    player_id: str = Field(min_length=1)
    color: str = Field(min_length=1)


class SubscribeGameRequest(BaseModel):
    game_id: str = Field(min_length=1)
    player_id: str | None = None
