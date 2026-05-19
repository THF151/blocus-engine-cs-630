# Backend Wire Protocol

> Wire contract between the FastAPI backend and any client (e.g. the Flutter
> frontend, the simulation script). Source-of-truth for action payloads and
> event shapes is the Pydantic schemas in `src/blocus_backend/schemas.py`;
> this document is maintained by hand alongside them.

This document covers the four supported modes: Classic two-, three-, and
four-player (20×20 board, blue/yellow/red/green) and Duo (14×14 board,
black/white, advanced-scoring-only).

---

## Connection Lifecycle & Seat Binding

### Endpoint

```
ws://<host>/ws
```

All traffic is JSON, framed by the WebSocket protocol. There is no HTTP
fallback. Every message from a client is a JSON object:

```json
{"action": "<action>", "payload": {...}}
```

The server replies with either an event (specific to the action) or an
error envelope (`{"type": "error", "code": "...", "message": "..."}`). The
server also broadcasts events to every connection subscribed to a game.

### Seat Binding

A connection becomes a **player** for a game by subscribing to it *with* a
`player_id`. A subscription without `player_id` puts the connection in
**spectator** mode for that game.

```jsonc
{
  "action": "subscribe_game",
  "payload": {
    "game_id": "game-42",
    "player_id": "alice"   // optional; omit for spectator mode
  }
}
```

- A connection may hold one seat per game and may be subscribed to several
  games simultaneously (different `(game_id, player_id)` bindings).
- Only the connection that holds seat `(game_id, player_id)` may submit
  `place_move` or `pass_move` for that pair. Mismatches produce
  `not_seated` or `player_mismatch` (see [Error Codes](#error-codes)).
- **Takeover.** If another connection subscribes to the same
  `(game_id, player_id)`, the previous holder is sent
  `{"type": "kicked", "reason": "seat_taken_by_reconnect"}` and its
  WebSocket is closed. This covers the browser-refresh case where the
  zombie connection has not yet been collected.
- Seat bindings are released when the connection disconnects.

### Multi-Worker Deployments

The seat binding map is held in-memory **per uvicorn worker**. Game state
itself is protected against concurrent writers by Lua-CAS in Redis
(`state.version` counter; see [Concurrency](#concurrency)). The
binding map is *not* propagated across workers.

Run behind a load balancer with **sticky sessions** keyed on the client
IP or a routing cookie so that reconnects land on the same worker:

- nginx: `ip_hash;` upstream directive
- Traefik: `loadBalancer.sticky.cookie`
- Kubernetes `Service`: `sessionAffinity: ClientIP`
- AWS ALB: target-group stickiness

Without sticky sessions, takeover semantics are best-effort: a reconnect
landing on a different worker will be granted the seat locally, but the
previous holder on the original worker is not evicted until its
connection times out.

---

## Action Catalog

Every action documented below lists:
- the Pydantic schema name (in `schemas.py`)
- the payload field table
- the success event the *caller* receives
- whether the success event is also broadcast to other subscribers
- error codes that may fire

### `create_game`

Creates a new game. The caller receives `game_created`; the same event is
broadcast to anyone who later subscribes (via `state_snapshot`). The caller
is **not** auto-bound to a seat; they must follow up with `subscribe_game`
including a `player_id` to play.

Payload (one of `TwoPlayerCreate` / `ThreePlayerCreate` / `FourPlayerCreate` /
`DuoCreateGameRequest`, discriminated by `mode`):

| Field        | Type                              | Required | Notes                                      |
|--------------|-----------------------------------|----------|--------------------------------------------|
| `mode`       | `"two_player"` \| `"three_player"` \| `"four_player"` \| `"duo"` | yes      |                                            |
| `game_id`    | string                            | no       | UUID generated if omitted                  |
| `scoring`    | `"basic"` \| `"advanced"`         | no       | Defaults to `"basic"`. For Duo, **must** be `"advanced"` (Literal-constrained; default is `"advanced"`). |
| `players`    | mode-specific (see below)         | yes      |                                            |
| `first_color`| mode-specific (see below)         | no       | Color whose player starts. See per-mode defaults below. |

**Player slot models:**

- `two_player`: `{"blue_red": "<id>", "yellow_green": "<id>"}` — each player controls two colors.
- `three_player`: `{"blue": "<id>", "yellow": "<id>", "red": "<id>", "shared_green": ["<id>", ...]}` — the green seat rotates through the listed players.
- `four_player`: `{"blue": "<id>", "yellow": "<id>", "red": "<id>", "green": "<id>"}`.
- `duo`: `{"black": "<id>", "white": "<id>"}`.

**`first_color` per mode:**

- `four_player`: one of `"blue"` / `"yellow"` / `"red"` / `"green"` (default `"blue"`). Rotates the four-color turn order so the chosen color goes first; the cycle continues clockwise.
- `duo`: one of `"black"` / `"white"` (default `"black"`). Same rotation logic against the two-color cycle.
- `two_player` / `three_player`: not configurable; the engine fixes the turn order.

Example:

```json
{
  "action": "create_game",
  "payload": {
    "game_id": "game-42",
    "mode": "four_player",
    "scoring": "advanced",
    "first_color": "red",
    "players": {"blue": "alice", "yellow": "bob", "red": "carol", "green": "dave"}
  }
}
```

Success event (broadcast): `game_created`.
Errors: `invalid_mode`, `invalid_classic_color`, `invalid_duo_color`, `invalid_players`, `invalid_scoring`, `missing_field`.

---

### `subscribe_game` (alias `join_game`)

Subscribes the connection to game events and optionally claims a seat.

Payload (`SubscribeGameRequest`):

| Field        | Type   | Required | Notes                                              |
|--------------|--------|----------|----------------------------------------------------|
| `game_id`    | string | yes      |                                                    |
| `player_id`  | string | no       | If set, claims `(game_id, player_id)` (may evict). |

Success event (unicast to caller): `state_snapshot`.
Side effect: if another connection holds the seat, it receives `kicked` and is closed.
Errors: `game_not_found`, `missing_field`.

---

### `request_state`

Re-fetches the current game state.

Payload: `{"game_id": "<id>"}`.
Success event (unicast): `state_snapshot`.
Errors: `game_not_found`, `missing_field`.

---

### `request_legal_moves`

Returns the legal moves for a `(player_id, color)` from the current state.
Does not require a seat binding (spectators may query).

Payload (`LegalMovesRequest`):

| Field       | Type   | Required |
|-------------|--------|----------|
| `game_id`   | string | yes      |
| `player_id` | string | yes      |
| `color`     | string | yes      |

Success event (unicast): `legal_moves`.
Errors: `game_not_found`, `invalid_classic_color`, `invalid_duo_color`, `missing_field`,
`invalid_command` (engine rejected the player/color combination).

---

### `place_move`

Places a piece. Requires the connection to hold the matching seat
`(game_id, player_id)`.

Payload (`PlaceMoveRequest`):

| Field            | Type   | Required | Notes                                         |
|------------------|--------|----------|-----------------------------------------------|
| `game_id`        | string | yes      |                                               |
| `command_id`     | string | yes      | Idempotency key for the engine                |
| `player_id`      | string | yes      | Must equal the connection's bound `player_id` |
| `color`          | string | yes      | Classic color owned by `player_id`            |
| `piece_id`       | int    | yes      |                                               |
| `orientation_id` | int    | yes      |                                               |
| `row`            | int    | yes      |                                               |
| `col`            | int    | yes      |                                               |

Success event (broadcast): `move_applied` (becomes `game_finished` when the
move ends the game).
Side effect: triggers `advance_ai_turns`, which may broadcast additional
AI-driven `move_applied` / `pass_applied` events.
Errors: `not_seated`, `player_mismatch`, `game_not_found`,
`invalid_classic_color`, `invalid_duo_color`, `missing_field`,
`rule_violation`, `invalid_command`, `conflict`.

---

### `pass_move`

Passes the current player's turn. Same seat requirement as `place_move`.

Payload (`PassMoveRequest`):

| Field        | Type   | Required |
|--------------|--------|----------|
| `game_id`    | string | yes      |
| `command_id` | string | yes      |
| `player_id`  | string | yes      |
| `color`      | string | yes      |

Success event (broadcast): `pass_applied`.
Errors: as `place_move`.

---

### `request_score`

Returns the current scoreboard.

Payload: `{"game_id": "<id>"}`.
Success event (unicast): `score_report`.
Errors: `game_not_found`, `missing_field`.

---

### `attach_ai`

Records a `(player_id, color)` pair as AI-controlled. The server's AI loop
will automatically play that color whenever it is current and no human is
bound to the seat (Binding ≻ AI). This does *not* require the caller to
hold the seat — any connection may attach an AI to any seat.

Payload (`AttachAiRequest`):

| Field       | Type   | Required |
|-------------|--------|----------|
| `game_id`   | string | yes      |
| `player_id` | string | yes      |
| `color`     | string | yes      |

Success event (broadcast): `game_joined`, followed by zero or more AI
`move_applied` / `pass_applied` events if it is now an AI's turn.
Errors: `game_not_found`, `invalid_classic_color`, `invalid_duo_color`,
`missing_field`, `conflict`.

---

## Event Catalog

All events except `error` and `kicked` carry the shape
`{"type": "<event_type>", "game_id": "<id>", "state": {...}, ...}`.

### `game_created` / `state_snapshot` / `move_applied` / `pass_applied` / `game_finished` / `game_joined`

All share the same `state` payload (the current state view). The
`type` distinguishes the cause; the `state` is always the latest:

```jsonc
{
  "type": "move_applied",
  "game_id": "game-42",
  "state": {
    "game_id": "game-42",
    "mode": "four_player",
    "scoring": "advanced",
    "status": "in_progress",        // or "finished"
    "version": 7,                    // server-managed write counter, monotonic
    "hash": 1234567890,              // engine state hash
    "board_size": 20,                // 14 for Duo
    "board_is_empty": false,
    "current_color": "yellow",
    "turn_order": ["red", "green", "blue", "yellow"],
    "occupied_count": 12,
    "board_counts": [{"color": "blue", "count": 3}, ...],
    "board_cells": [{"row": 0, "col": 0, "color": "blue"}, ...], // all occupied cells
    "shared_color_turn_index": 1     // only present in three_player
  },
  "response": "move applied"         // move_applied/pass_applied/game_finished only
}
```

- `move_applied` becomes `game_finished` (same payload shape) on the move
  that ends the game.
- `game_joined` is emitted on `attach_ai`.

### `player_joined`

Broadcast to all subscribers when a player claims a seat via `subscribe_game`
with a `player_id`. Allows lobby UIs to show who has joined without polling.

```json
{
  "type": "player_joined",
  "game_id": "game-42",
  "player_id": "alice",
  "state": { ... }
}
```

The joining player also receives this broadcast, followed by a unicast
`state_snapshot`. Clients may use whichever they prefer.

### `legal_moves`

```json
{
  "type": "legal_moves",
  "game_id": "game-42",
  "player_id": "alice",
  "color": "blue",
  "moves": [
    {"piece_id": 0, "orientation_id": 0, "row": 0, "col": 0, "board_index": 0, "score_delta": 1}
  ]
}
```

### `score_report`

```json
{
  "type": "score_report",
  "game_id": "game-42",
  "score": {
    "scoring": "advanced",
    "entries": [{"player_id": "alice", "score": 87}, {"player_id": "bob", "score": 71}]
  }
}
```

### `kicked`

Sent to the previous holder of a seat when another connection takes it
over via `subscribe_game`. The WebSocket is closed immediately after.

```json
{"type": "kicked", "reason": "seat_taken_by_reconnect"}
```

### `error`

```json
{"type": "error", "code": "<error_code>", "message": "<human-readable>"}
```

The `message` is informative for debugging only — **clients should branch
on `code`**, not on message text. See [Error Codes](#error-codes).

---

## Error Codes

| Code                       | When                                                                                 | Client action                                                  |
|----------------------------|--------------------------------------------------------------------------------------|----------------------------------------------------------------|
| `invalid_message`          | Top-level message is not a JSON object, or `payload` is not an object.               | Fix request shape.                                             |
| `unknown_action`           | `action` is not in the action catalog.                                               | Fix `action` value.                                            |
| `missing_field`            | A required field is missing or has the wrong primitive type.                         | Inspect `message` for the offending field path.                |
| `invalid_players`          | The `players` block fails structural validation (missing slot, wrong shape).         | Fix the `players` payload to match the mode-specific schema.   |
| `invalid_mode`             | `mode` is not one of `two_player` / `three_player` / `four_player` / `duo`.          | Use a supported mode.                                          |
| `invalid_classic_color`    | A color string is not one of the Classic colors (`blue` / `yellow` / `red` / `green`) in a Classic game. | Send a color valid for the game's mode.                        |
| `invalid_duo_color`        | A color string is not `black` / `white` in a Duo game.                               | Send a color valid for the game's mode.                        |
| `invalid_scoring`          | `scoring` is not `basic` / `advanced`, or `basic` was sent for a Duo game (Duo is advanced-only). | Send a supported scoring mode.                                 |
| `game_not_found`           | The `game_id` does not exist in Redis.                                               | Surface to user; offer to recreate or rejoin.                  |
| `not_seated`               | `place_move` / `pass_move` sent from a connection that hasn't claimed this game's seat. | Send `subscribe_game` with `player_id` first.                |
| `player_mismatch`          | The bound `player_id` for this connection's seat differs from the move's `player_id`. | Either re-subscribe with the correct `player_id` or stop forging the field. |
| `rule_violation`           | The engine rejected the move because it breaks Blokus rules.                         | Surface error to user; re-render board. The engine `message` is descriptive but the schema may change. |
| `invalid_command`          | The engine rejected command parameters (unknown piece_id, etc.).                     | Check command parameters against the latest state.             |
| `conflict`                 | Concurrent write detected. Another writer's version won this CAS round.              | Re-fetch state via `request_state` and let the user retry.     |
| `ai_turn_limit_exceeded`   | The AI loop hit its safety cap (`MAX_AI_TURNS=10_000`).                              | Treat as `internal_error`; report bug to backend team.         |
| `internal_error`           | Unexpected server-side error. Details are server-logged, not returned.               | Surface generic error; offer retry.                            |

---

## Concurrency

State writes go through an optimistic concurrency check:

- Every persisted game state carries a **server-managed write counter**
  (`state.version` in events). The counter is incremented by Redis Lua-CAS
  on every save (move *or* metadata, e.g. `attach_ai`).
- Clients **do not** send `expected_version` — the server tracks it
  internally. Clients only need to handle the `conflict` error code.
- On `conflict`, the server has **not** applied the move. The client should
  re-fetch state (`request_state` or wait for the next broadcast) and
  re-prompt the user. The server never auto-retries human moves.
- The AI loop *does* retry internally on `conflict` (re-reads state and
  re-derives the move).

State writes from multiple uvicorn workers are safely serialized via the
Lua-CAS script in `repository.py`. Pub/Sub fan-out keeps subscribers on
all workers in sync.

---

## Trust Model

The backend treats the WebSocket connection itself as the unit of identity:

- **Authenticated:** no. There is no token, cookie, or session. Anyone
  who can reach `/ws` may connect and subscribe.
- **Per-connection seat binding** prevents a connected client from
  submitting moves as a *different* `player_id` than the one they claimed.
  This rules out buggy and casually malicious clients forging the
  `player_id` field on `place_move`.
- **Transport security** is a deployment concern: terminate TLS at the
  load balancer / reverse proxy (nginx, Traefik, ALB). The protocol
  carries no secrets.
- **No move forgery via MITM** is out of scope — that would require true
  authentication (e.g. JWTs bound to the connection). The grading rubric
  in `deliverables/team/OWNERSHIP.md` does not require it; if it ever
  does, an authentication step can be added at `subscribe_game` without
  reshaping the rest of the protocol.

---

## Versioning

This protocol is **v1** and currently unversioned on the wire — clients do
not need to send a version header.

If a backwards-incompatible change becomes necessary, the new wire format
will be reachable at `/ws/v2` and `/ws` will continue to serve v1 for a
deprecation period. Additive changes (new actions, new optional fields,
new error codes for already-failing paths) are made in-place on v1.

---

## Out of Scope

These are deliberate gaps, not oversights:

- **Authentication tokens.** See [Trust Model](#trust-model).
- **Move clocks.** The engine does not expose turn timers.
- **Rate limiting.** Backend trusts the deployment to handle abuse.
- **Cross-worker takeover.** Mitigated by sticky sessions; see
  [Multi-Worker Deployments](#multi-worker-deployments).
