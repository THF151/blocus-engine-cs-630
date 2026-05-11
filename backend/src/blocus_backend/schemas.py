"""Pydantic models for the websocket protocol.

These define the *structural* contract for each action's payload (required
fields, primitive types, list shape). Domain-level validation that needs a
granular wire error code lives in ``service.py``:

- mode set membership → ``invalid_classic_mode``
- color set membership → ``invalid_classic_color``
- scoring set membership → ``invalid_scoring``

Pydantic ``ValidationError`` is mapped to ``invalid_players`` (when the
failure is under the ``players`` field) or ``missing_field`` otherwise.
"""

from __future__ import annotations

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
