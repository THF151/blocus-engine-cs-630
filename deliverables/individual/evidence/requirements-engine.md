# Blocus Engine — Core Requirements

## 1. Scope

This document specifies the requirements for the pure Rust Blokus engine and the Python binding layer that exposes the engine to the backend.

The engine is responsible for the full game lifecycle of Classic Blokus and Blokus Duo:

- game configuration
- state initialization
- board and piece state management
- placement validation
- legal move generation
- turn progression
- pass handling
- game termination
- scoring
- deterministic state hashing
- safe transfer of state through the Python binding layer

Classic Blokus must support two-player, three-player, and four-player variants on a 20x20 board.  
Blokus Duo must support two players on a 14x14 board.

The following areas are out of scope for this document:

- persistence
- networking
- matchmaking
- frontend UI and UX
- AI move-selection policy
- production deployment

The engine must expose primitives that allow those layers to use it, but the pure engine must not depend on those layers.

---

## 2. Modal Verb Conventions

This document uses the following requirement words.

| Verb | Meaning | Violation Consequence |
|---|---|---|
| **MUST** | Correctness rule. The engine depends on this always being true. | Engine bug. |
| **SHALL** | Required feature or public capability. | Missing feature. |
| **SHOULD** | Quality goal or recommended design choice. | Acceptable short-term limitation, but should be tracked. |

Optional behavior is not specified in this version.

---

## 3. Glossary

- **Active color** — A color that belongs to the selected game mode. Classic uses the four classic colors. Duo uses the two Duo colors.
- **Anchor** — The board position where the local top-left position of a piece orientation is placed.
- **Corner contact** — Two same-color cells touch diagonally at one corner.
- **Edge contact** — Two same-color cells touch directly by one side.
- **Opening target** — The required start cell or start cells for the first move of a color.
- **Orientation** — A unique rotated or mirrored form of a Blokus piece.
- **Padded-row indexing** — An internal board layout where every row has extra unused bits to make bit operations faster.
- **Placement** — A concrete combination of piece, orientation, and board anchor.
- **Snapshot iterator** — A legal move iterator that copies the needed state at construction time and does not borrow the live game state.
- **Position hash** — A deterministic value that identifies a game position based on board state, inventory state, turn state, and game topology.

---

## 4. Functional Requirements

### 4.1 Domain Model and State Representation

#### REQ-CORE-001 — Pure domain layer

**Statement:**  
The pure Rust engine **MUST NOT** depend on Python, backend web frameworks, Redis, WebSockets, Flutter, or AI/ML libraries.

**Rationale:**  
The core engine should be reusable, testable, and independent from transport, storage, UI, and AI layers.

**Acceptance Criteria:**

- The core engine dependency list contains only domain-safe dependencies.
- The core engine builds without Python, HTTP, Redis, Flutter, or AI dependencies.
- The Python binding layer is separate from the pure engine logic.

**Dependencies:**  
None.

---

#### REQ-CORE-002 — Compact board representation

**Statement:**  
Board occupancy **MUST** use a compact fixed-size bitmask representation with padded-row indexing.

**Rationale:**  
The engine frequently needs to combine, compare, and shift board states. A compact mask representation allows fast set operations for move validation and move generation.

**Acceptance Criteria:**

- The board representation supports union, intersection, difference, and directional shifts.
- The board representation can compute edge-neighbor and diagonal-neighbor cells.
- Invalid padding bits are rejected at validation or deserialization boundaries.
- Board masks can be converted back into playable board cells in deterministic order.

**Dependencies:**  
None.

---

#### REQ-CORE-003 — Canonical piece repository

**Statement:**  
The engine **SHALL** expose the full official set of 21 Blokus pieces, each with all unique orientations precomputed.

**Rationale:**  
Piece shapes and orientations are fixed by the game rules. They should be generated once and reused by placement validation, move generation, scoring, and UI-facing APIs.

**Acceptance Criteria:**

- Exactly 21 canonical pieces are available.
- Every piece exposes its shape size and number of occupied cells.
- Every piece exposes only unique orientations.
- No piece exposes more than the maximum number of possible rotation and mirror combinations.
- The Python binding can expose pieces and orientations to backend consumers.

**Dependencies:**  
REQ-CORE-002.

---

#### REQ-CORE-004 — Multi-mode board geometry

**Statement:**  
The engine **SHALL** support board geometry for Classic Blokus and Blokus Duo.

Classic modes **SHALL** use a 20x20 playable board.  
Duo mode **SHALL** use a 14x14 playable board.

**Rationale:**  
The project supports more than one Blokus variant. Board size and opening rules must be derived from the selected game mode, not hard-coded for only Classic.

**Acceptance Criteria:**

- The selected game mode determines the board size.
- Classic modes use the classic board.
- Duo mode uses the Duo board.
- The public state view returns the correct board size.
- Move validation rejects placements outside the active board.

**Dependencies:**  
REQ-CORE-002, REQ-CORE-003.

---

#### REQ-CORE-005 — Safe Rust only

**Statement:**  
The Rust workspace **MUST** forbid unsafe code in the core engine and Python binding layer.

**Rationale:**  
The engine should keep Rust's memory safety guarantees. The domain logic does not require unsafe code.

**Acceptance Criteria:**

- Unsafe code is forbidden by workspace lint configuration.
- The project builds and lints successfully with this restriction.
- No implementation requires unsafe blocks.

**Dependencies:**  
None.

---

### 4.2 Game Configuration

#### REQ-CFG-001 — Mode policy validation

**Statement:**  
Game configuration validation **MUST** reject any setup where the selected player ownership model does not match the selected game mode or the turn order violates the selected game mode.

**Rationale:**  
Invalid game setup can corrupt later state transitions. It must be rejected before the game starts.

**Acceptance Criteria:**

- Two-player and three-player Classic modes require the official fixed color order.
- Four-player Classic mode allows only clockwise rotations of the official color order.
- Duo mode allows only alternating Duo colors.
- Invalid mode and turn-order combinations produce a structured input error.
- Valid configurations produce an initialized game state.

**Dependencies:**  
REQ-CORE-004.

---

#### REQ-CFG-002 — Two-player ownership

**Statement:**  
Two-player Classic mode **SHALL** assign two colors to one player and the remaining two colors to the other player. The two assigned players **MUST** be distinct.

**Rationale:**  
This matches the official two-player Classic Blokus ownership model.

**Acceptance Criteria:**

- A valid two-player setup creates two distinct player ownership groups.
- A setup using the same player for both ownership groups is rejected.
- Turn ownership checks follow the configured color ownership.

**Dependencies:**  
REQ-CFG-001.

---

#### REQ-CFG-003 — Three-player shared color

**Statement:**  
Three-player mode **SHALL** assign three individually owned colors to three distinct players and one shared color whose ownership rotates between those same players.

**Rationale:**  
This matches the official three-player variant and requires turn ownership to depend on the shared-color turn index.

**Acceptance Criteria:**

- The shared color cannot also be individually owned.
- The shared-color rotation cannot contain duplicate players.
- The shared-color rotation must contain the same players that own the individual colors.
- The correct shared-color owner is used during placement validation.
- A wrong shared-color owner is rejected.

**Dependencies:**  
REQ-CFG-001.

---

#### REQ-CFG-004 — Four-player ownership

**Statement:**  
Four-player Classic mode **SHALL** assign exactly one classic color to each player. Duplicate colors or duplicate player assignments **MUST** be rejected.

**Rationale:**  
Each player controls exactly one color in the official four-player mode.

**Acceptance Criteria:**

- A valid four-player setup assigns one color per player.
- Duplicate color assignments are rejected.
- Duplicate player assignments are rejected.
- The turn order must preserve clockwise order, allowing only a rotated starting color.

**Dependencies:**  
REQ-CFG-001.

---

#### REQ-CFG-005 — Duo configuration

**Statement:**  
Duo mode **SHALL** use two Duo colors, a 14x14 board, and two fixed shared starting points. Duo mode **MUST** reject Basic scoring.

**Rationale:**  
Duo uses different colors, board size, and opening rules from Classic. The project defines Duo as advanced-scoring only.

**Acceptance Criteria:**

- Duo setup creates a two-player game with the Duo colors.
- Duo setup uses the Duo board size.
- Duo setup defaults to advanced scoring.
- Explicit Basic scoring for Duo is rejected.
- First moves in Duo must cover one of the shared starting points.
- After the first Duo player uses one start, the second player's first move must use the remaining start.

**Dependencies:**  
REQ-CFG-001, REQ-CORE-004.

---

### 4.3 Game Initialization

#### REQ-INIT-001 — Deterministic initialization

**Statement:**  
Game initialization **MUST** produce a deterministic initial state with:

- empty board
- empty inventories
- no recorded last piece
- initial state version
- active game status
- first color from the configured turn order
- deterministic position hash

**Rationale:**  
A deterministic initial state is required for replay, state comparison, hashing, testing, and JSON round trips.

**Acceptance Criteria:**

- The initialized state has an empty board.
- No pieces are marked as used.
- The active color matches the configured first color.
- The state version starts at the initial value.
- The game status is active.
- The initial hash is non-zero and reproducible.
- Initialization works for two-player, three-player, four-player, and Duo modes.

**Dependencies:**  
REQ-CFG-001 through REQ-CFG-005, REQ-HASH-001.

---

### 4.4 Placement Rules

#### REQ-RULE-001 — Bounds enforcement

**Statement:**  
Placement validation **MUST** reject any placement outside the active board geometry.

**Rationale:**  
Classic and Duo have different board sizes. A placement that is legal on the physical storage layout may still be illegal for the selected game mode.

**Acceptance Criteria:**

- Classic placements are limited to the Classic board.
- Duo placements are limited to the Duo board.
- Placements outside the active board return an out-of-bounds rule violation.
- State validation rejects occupied cells outside the active board.

**Dependencies:**  
REQ-CORE-002, REQ-CORE-004.

---

#### REQ-RULE-002 — Overlap rejection

**Statement:**  
Placement validation **MUST** reject any placement that overlaps an already occupied board cell.

**Rationale:**  
Two pieces cannot occupy the same cell.

**Acceptance Criteria:**

- A placement overlapping any existing piece is rejected.
- The rejection uses a structured overlap rule violation.
- Board state is not mutated after a rejected placement.

**Dependencies:**  
REQ-CORE-002, REQ-RULE-001.

---

#### REQ-RULE-003 — Same-color edge contact forbidden

**Statement:**  
For non-opening placements, a piece **MUST NOT** share an edge with a piece of the same color.

**Rationale:**  
This is a core Blokus rule.

**Acceptance Criteria:**

- Same-color edge contact is detected.
- Same-color edge contact is rejected with a structured rule violation.
- Edge contact with other colors does not trigger this same-color rule.

**Dependencies:**  
REQ-CORE-002.

---

#### REQ-RULE-004 — Same-color corner contact required

**Statement:**  
For non-opening placements, a piece **MUST** touch at least one same-color piece by corner.

**Rationale:**  
This is a core Blokus rule and is required after the first move of a color.

**Acceptance Criteria:**

- A non-opening placement with same-color corner contact can be accepted if all other rules pass.
- A non-opening placement without same-color corner contact is rejected.
- The rejection uses a structured missing-corner-contact rule violation.

**Dependencies:**  
REQ-CORE-002.

---

#### REQ-RULE-005 — Opening placement target

**Statement:**  
The first placement of each color **MUST** cover the opening target for the selected mode.

Classic opening moves must cover the assigned corner for that color.  
Duo opening moves must cover one of the two shared starting points.

**Rationale:**  
Opening moves differ between Classic and Duo and must be enforced by the rules layer.

**Acceptance Criteria:**

- The first Classic move for each color must cover that color's assigned corner.
- The first Duo move must cover one of the shared starting points.
- The second Duo color's first move must cover the remaining shared starting point.
- An opening move that misses the required target is rejected.

**Dependencies:**  
REQ-CORE-004.

---

#### REQ-RULE-006 — Inventory enforcement

**Statement:**  
Placement validation **MUST** reject any attempt to place a piece that the placing color has already used.

**Rationale:**  
Each color owns exactly one copy of each official Blokus piece.

**Acceptance Criteria:**

- After a successful placement, the placed piece is marked as used for that color.
- Used pieces no longer appear in that color's available inventory.
- Reusing a piece for the same color is rejected.
- Other colors still have their own independent inventory.

**Dependencies:**  
REQ-CORE-003.

---

#### REQ-RULE-007 — Turn and ownership validation

**Statement:**  
Placement validation **MUST** reject submissions where the submitted color is not the current turn color or where the submitting player does not control that color in the current turn context.

**Rationale:**  
The engine must prevent out-of-turn actions and actions by players who do not control the submitted color.

**Acceptance Criteria:**

- A command for the wrong color is rejected.
- A command from a player who does not control the submitted color is rejected.
- Shared-color ownership in three-player mode is evaluated using the current shared-color rotation.
- Rejections use structured rule violation errors.

**Dependencies:**  
REQ-CFG-001 through REQ-CFG-005, REQ-TURN-004.

---

#### REQ-RULE-008 — Reject commands after game end

**Statement:**  
The engine **MUST** reject any place or pass command applied after the game has finished.

**Rationale:**  
No game actions are legal after termination.

**Acceptance Criteria:**

- A placement command after termination is rejected.
- A pass command after termination is rejected.
- The rejection uses a structured game-already-finished rule violation.
- The finished state is not mutated by rejected commands.

**Dependencies:**  
REQ-END-001.

---

### 4.5 Move Generation

#### REQ-MOVE-001 — Lazy snapshot iterator

**Statement:**  
The engine **SHALL** provide a lazy legal move iterator that does not borrow the live game state and yields legal moves one at a time.

**Rationale:**  
Callers such as the UI, backend agents, and future search code may only need the first legal move or a small subset of moves. They should not be forced to materialize all moves first.

**Acceptance Criteria:**

- The iterator can be created from a game state, player, and color.
- The iterator owns or copies the state-derived data it needs.
- The iterator yields valid moves one by one.
- Public helper methods can also collect all moves when needed.
- The Python binding exposes collected legal move queries.

**Dependencies:**  
REQ-CORE-003.

---

#### REQ-MOVE-002 — Stable iteration order

**Statement:**  
Legal move iteration order **MUST** be deterministic and stable.

**Rationale:**  
Stable ordering supports reproducible tests, deterministic AI behavior, and reliable debugging.

**Acceptance Criteria:**

- Moves are ordered consistently by piece, orientation, and board position.
- Repeating the same query on the same state produces the same order.
- The order is consistent across Rust and Python-facing move queries.

**Dependencies:**  
REQ-MOVE-001.

---

#### REQ-MOVE-003 — Diagonal-frontier optimization

**Statement:**  
When a color already has occupied cells, move generation **SHOULD** restrict candidate anchors to the same-color diagonal frontier instead of scanning the entire board.

**Rationale:**  
This keeps legal move generation faster while preserving the same legal result.

**Acceptance Criteria:**

- Non-opening move generation uses same-color diagonal frontier information.
- Correctness is preserved compared to the full rule set.
- Future benchmarks should compare this approach against a brute-force reference implementation.

**Dependencies:**  
REQ-CORE-002, REQ-MOVE-001.

---

#### REQ-MOVE-004 — Empty iterator for invalid move context

**Statement:**  
The legal move iterator **MUST** yield no moves when the context is invalid.

Invalid contexts include:

- finished game
- wrong color's turn
- player not scheduled to control the requested color

**Rationale:**  
Move consumers should be able to ask for legal moves safely without crashing or pre-validating every condition.

**Acceptance Criteria:**

- Finished states return no legal moves.
- Wrong-turn queries return no legal moves.
- Queries by a player who does not control the color return no legal moves.
- Boolean legal-move checks follow the same behavior.

**Dependencies:**  
REQ-MOVE-001.

---

#### REQ-MOVE-005 — Derived move query helpers

**Statement:**  
The engine **SHALL** expose helper queries for:

- all valid moves
- valid moves for one piece
- whether any valid move exists
- whether any valid move exists for one piece

**Rationale:**  
These are common backend, UI, and AI queries. They should be derived from the same legal move logic to avoid inconsistent behavior.

**Acceptance Criteria:**

- All helper queries use the same rule source as the legal move iterator.
- Piece-specific queries only return moves for the requested piece.
- Boolean queries return true only when at least one matching legal move exists.
- The Python binding exposes these helpers.

**Dependencies:**  
REQ-MOVE-001.

---

### 4.6 Turn Order and Passing

#### REQ-TURN-001 — Cycle through unpassed active colors

**Statement:**  
Turn advancement **MUST** select the next active color that has not permanently passed. If all active colors have passed, turn advancement must signal that no next turn exists.

**Rationale:**  
This is required for pass behavior and game termination.

**Acceptance Criteria:**

- Passed colors are skipped during turn advancement.
- Only active colors for the selected game mode are considered.
- If no active unpassed color remains, advancement signals no next turn.
- Turn advancement preserves the selected mode's configured order.

**Dependencies:**  
REQ-CFG-001.

---

#### REQ-TURN-002 — Permanent pass

**Statement:**  
Once a color passes, that color **MUST** remain passed for the rest of the game.

**Rationale:**  
Blokus does not allow a color to re-enter after passing.

**Acceptance Criteria:**

- Passing marks the color in a persistent passed-color tracker.
- Passed colors are skipped on future turn advancement.
- A passed color is never selected again as the current color.
- The passed-color tracker only grows during a game.

**Dependencies:**  
REQ-TURN-001.

---

#### REQ-TURN-003 — Pass only when blocked

**Statement:**  
A pass command **MUST** be rejected if the current color has at least one legal move.

**Rationale:**  
Passing is only allowed when a player is blocked. This prevents intentional early passing and keeps termination behavior correct.

**Acceptance Criteria:**

- The engine checks for legal moves before accepting a pass.
- If at least one legal move exists, the pass is rejected.
- The rejection uses a structured pass-not-allowed rule violation.
- The state is not mutated after a rejected pass.

**Dependencies:**  
REQ-MOVE-001.

---

#### REQ-TURN-004 — Three-player shared-color rotation

**Statement:**  
When the shared color in three-player mode completes a turn, the shared-color ownership index **MUST** advance to the next player in the shared-color cycle.

**Rationale:**  
The shared color must rotate between the three players according to the official three-player variant.

**Acceptance Criteria:**

- Shared-color placement advances the shared-color owner index.
- Shared-color pass advances the shared-color owner index.
- The next shared-color turn belongs to the next player in the configured cycle.
- A player outside the current shared-color turn is rejected.

**Dependencies:**  
REQ-CFG-003.

---

### 4.7 Game Termination

#### REQ-END-001 — Termination condition

**Statement:**  
After every successful command, the engine **MUST** finish the game if no active, non-passed color has any legal move.

**Rationale:**  
The game must end when all remaining active colors are blocked or passed.

**Acceptance Criteria:**

- After a successful placement or pass, the engine checks whether any active unpassed color still has a legal move.
- If none do, the game status becomes finished.
- If at least one active unpassed color can still move, the game continues.
- Finished games can be scored.

**Dependencies:**  
REQ-MOVE-001, REQ-TURN-001.

---

#### REQ-END-002 — Termination event and response

**Statement:**  
A command that finishes the game **MUST** return a game-finished event and a game-finished response summary.

**Rationale:**  
Backend and UI layers need a clear signal to stop normal turn handling and move to final scoring.

**Acceptance Criteria:**

- A terminating command returns a game-finished event.
- A terminating command returns a game-finished response summary.
- A non-terminating command returns a normal move or pass response and a turn-advanced event.
- Event versioning matches the resulting state version.

**Dependencies:**  
REQ-END-001.

---

### 4.8 Scoring

#### REQ-SCORE-001 — Basic scoring

**Statement:**  
Basic scoring **SHALL** return the number of unplayed squares for each relevant color or player. Lower is better.

**Rationale:**  
This engine exposes Basic scoring as a remaining-square count. This is different from Advanced scoring, which uses signed points.

**Acceptance Criteria:**

- Remaining piece squares are counted correctly.
- Basic scoring reports remaining squares.
- Two-player scores are aggregated by player ownership.
- Scoring before game termination is rejected.

**Dependencies:**  
REQ-RULE-006.

---

#### REQ-SCORE-002 — Advanced scoring

**Statement:**  
Advanced scoring **SHALL** subtract one point per remaining square. It **MUST** award a completion bonus when a color places all pieces and a larger bonus if the final piece was the one-square piece.

**Rationale:**  
This implements the advanced Blokus scoring rule, including the last-piece bonus.

**Acceptance Criteria:**

- Incomplete inventories score negatively by remaining square count.
- A color that uses all pieces receives the completion bonus.
- A color that uses all pieces and finishes with the one-square piece receives the larger completion bonus.
- Last-piece tracking is updated after successful placements.

**Dependencies:**  
REQ-RULE-006.

---

#### REQ-SCORE-003 — Two-player score aggregation

**Statement:**  
Two-player scoring **MUST** aggregate each player's two controlled colors into one player score.

**Rationale:**  
In the two-player variant, each player controls two colors.

**Acceptance Criteria:**

- The first player's two colors are summed into one score entry.
- The second player's two colors are summed into one score entry.
- No duplicate player score entries are returned for the same player.
- Aggregation works for both Basic and Advanced scoring.

**Dependencies:**  
REQ-CFG-002.

---

#### REQ-SCORE-004 — Three-player shared-color exclusion

**Statement:**  
Three-player scoring **MUST** ignore the shared color and score only the individually owned colors.

**Rationale:**  
The shared color is not owned permanently by one player, so final scoring should use only individually owned colors.

**Acceptance Criteria:**

- The shared color does not produce a score entry.
- Each individually owned color contributes to its owning player.
- The scoreboard contains one score entry per permanent player.
- The scoring behavior is independent of the current shared-color turn index.

**Dependencies:**  
REQ-CFG-003.

---

#### REQ-SCORE-005 — Reject scoring before termination

**Statement:**  
The engine **MUST** reject final scoring requests while the game is still active.

**Rationale:**  
Final scores are only meaningful after the game has finished.

**Acceptance Criteria:**

- Scoring an active game returns a structured game-not-finished rule violation.
- Scoring a finished game returns a scoreboard.
- Rejected scoring requests do not mutate state.

**Dependencies:**  
REQ-END-001.

---

#### REQ-SCORE-006 — Duo scoring restriction

**Statement:**  
Duo scoring **MUST** use Advanced scoring only.

**Rationale:**  
The project defines Duo as advanced-scoring only to keep Duo scoring consistent across configuration, state loading, and final score calculation.

**Acceptance Criteria:**

- Duo game configuration rejects Basic scoring.
- Scoring a Duo state with Basic scoring is rejected.
- Loading a Duo state with an invalid scoring mode is rejected.
- Valid Duo scoring produces Advanced scoring results.

**Dependencies:**  
REQ-CFG-005, REQ-SCORE-002.

---

### 4.9 State Determinism

#### REQ-HASH-001 — Deterministic full hash

**Statement:**  
The full position hash **MUST** be a pure function of the observable game position.

It must not depend on:

- stored hash field itself
- game identifier
- schema version
- command identifiers
- state version

**Rationale:**  
Equivalent positions should produce the same hash even if they occur in different games or at different version numbers.

**Acceptance Criteria:**

- Recomputing the hash for the same position produces the same value.
- Loading a state from JSON recomputes the canonical hash.
- Changing only excluded metadata does not change the position hash.
- The hash is stable across Rust and Python state round trips.

**Dependencies:**  
None.

---

#### REQ-HASH-002 — Incremental hash transitions

**Statement:**  
After every successful command, the resulting state's stored hash **MUST** equal the full recomputed position hash. The implementation **SHOULD** use incremental hash updates on hot paths.

**Rationale:**  
The hash must be correct, and incremental updates avoid unnecessary full-board recomputation during normal command application.

**Acceptance Criteria:**

- Successful placement produces a state with a correct hash.
- Successful pass produces a state with a correct hash.
- JSON round trip preserves the canonical hash.
- Incremental hash logic is validated against full recomputation.

**Dependencies:**  
REQ-HASH-001.

---

#### REQ-HASH-003 — Monotonic state version

**Statement:**  
The state version **MUST** increase after every successful command.

**Rationale:**  
The backend can use the version for optimistic concurrency and stale-state detection.

**Acceptance Criteria:**

- A successful placement increments the state version.
- A successful pass increments the state version.
- Rejected commands do not create a new state version.
- Version increase is deterministic and monotonic.

**Dependencies:**  
None.

---

### 4.10 Error Handling

#### REQ-ERR-001 — Three-category error hierarchy

**Statement:**  
All public engine errors **MUST** belong to exactly one of these categories:

- rule violation
- input error
- engine error

**Rationale:**  
Consumers need to distinguish between illegal player actions, malformed caller input, and engine/state corruption.

**Acceptance Criteria:**

- Rule violations are used for illegal game actions.
- Input errors are used for malformed or inconsistent caller input.
- Engine errors are used for corrupted state or internal invariant failures.
- Every public error exposes its category.

**Dependencies:**  
None.

---

#### REQ-ERR-002 — Stable error codes

**Statement:**  
Each public error variant **MUST** expose a stable code and human-readable message. Error codes **MUST NOT** be renamed without a schema/version change.

**Rationale:**  
Backend and frontend layers can use stable codes for client responses, UI messages, and debugging.

**Acceptance Criteria:**

- Each error exposes a category, code, and message.
- Python exceptions preserve the structured error information.
- Tests check representative input errors and rule violations.
- Error messages are suitable for logs and user-facing conversion.

**Dependencies:**  
REQ-ERR-001.

---

#### REQ-ERR-003 — Corrupted state detection

**Statement:**  
Any state that violates board, color, turn, inventory, mode, or Duo consistency invariants **MUST** be rejected as corrupted or invalid.

**Rationale:**  
Deserialized state and externally supplied state are trust boundaries. The engine must not silently accept corrupted state.

**Acceptance Criteria:**

- Board masks with invalid padding bits are rejected.
- Overlapping color masks are rejected.
- Inactive colors cannot contain occupied cells or used pieces.
- Invalid turn state is rejected.
- Inconsistent Duo state is rejected.
- Corruption is reported through a structured engine error or input error as appropriate.

**Dependencies:**  
REQ-CORE-002, REQ-CFG-001.

---

## 5. Interface Requirements

### 5.1 Rust Core API

#### REQ-API-001 — Stateless engine facade

**Statement:**  
The Rust engine facade **MUST** be stateless. It must not own game state and should only reference shared immutable game data.

**Rationale:**  
A stateless engine can safely drive many games and is easier to use from backend services and future AI/search code.

**Acceptance Criteria:**

- Creating an engine does not create a game state.
- Game state is passed into engine operations explicitly.
- Applying a command returns a new state instead of mutating the old one.
- The engine can be copied or reused across games.

**Dependencies:**  
REQ-CORE-003.

---

#### REQ-API-002 — Typed identifiers

**Statement:**  
The Rust core **SHALL** use typed identifiers for games, players, commands, pieces, orientations, board positions, state versions, and position hashes.

**Rationale:**  
Typed identifiers reduce the risk of mixing unrelated values and keep the state model explicit.

**Acceptance Criteria:**

- Game, player, and command identifiers are represented as typed wrappers.
- Piece and orientation identifiers reject out-of-range values.
- Board position identifiers reject invalid board coordinates.
- State version and position hash values use dedicated types.
- The Python binding maps invalid identifiers to structured input errors.

**Dependencies:**  
REQ-ERR-001.

---

### 5.2 Python Binding Layer

#### REQ-FFI-001 — Surface parity

**Statement:**  
The Python binding layer **SHALL** expose the engine operations, configuration objects, command objects, state objects, result objects, piece objects, scoring objects, event objects, response objects, and enum-like value classes needed by the backend.

**Rationale:**  
The backend should be able to drive the complete game lifecycle without implementing game rules itself.

**Acceptance Criteria:**

- The Python binding can initialize games.
- The Python binding can apply place and pass commands.
- The Python binding can query legal moves.
- The Python binding can inspect board state, inventories, pieces, and scores.
- The Python binding exposes structured result, event, and response objects.
- The Python binding supports Classic and Duo configuration.

**Dependencies:**  
REQ-API-001, REQ-API-002.

---

#### REQ-FFI-002 — Structured Python exceptions

**Statement:**  
Python-facing errors **MUST** inherit from one common base exception and provide concrete subclasses for input errors, rule violations, and engine errors.

Each exception message **MUST** preserve category, code, and message information.

**Rationale:**  
Backend code should be able to catch errors by category and convert them into client responses.

**Acceptance Criteria:**

- There is one common base exception.
- There are separate subclasses for input errors, rule violations, and engine errors.
- Structured category, code, and message information is preserved.
- Invalid input, illegal moves, and corrupted state map to the correct Python exception category.

**Dependencies:**  
REQ-ERR-001, REQ-ERR-002.

---

#### REQ-FFI-003 — Value semantics

**Statement:**  
Python-facing value objects **MUST** be immutable from Python and compare by content.

**Rationale:**  
This makes the Python API predictable for tests, backend logic, sets, dictionaries, and equality checks.

**Acceptance Criteria:**

- Command objects compare by their field values.
- Enum-like value classes compare by their logical value.
- Result and state helper objects expose stable values.
- Python tests can compare objects directly in assertions.

**Dependencies:**  
REQ-FFI-001.

---

### 5.3 JSON Persistence

#### REQ-JSON-001 — Stable state round trip

**Statement:**  
The Python binding **SHALL** provide JSON serialization and deserialization for game state. A state produced by the engine must survive a JSON round trip without losing game-relevant information.

**Rationale:**  
The backend persistence layer needs a stable state format for storage and restoration.

**Acceptance Criteria:**

- Initial states can be serialized and loaded again.
- States after moves can be serialized and loaded again.
- Classic and Duo states round trip correctly.
- The loaded state has the same canonical position hash as the original.
- Public state fields remain consistent after loading.

**Dependencies:**  
REQ-HASH-001.

---

#### REQ-JSON-002 — Hash recomputation on load

**Statement:**  
State loading **MUST** ignore any supplied stored hash value and recompute the canonical position hash.

**Rationale:**  
External JSON cannot be trusted. Recomputing the hash prevents stale or tampered hash values from entering the engine.

**Acceptance Criteria:**

- Changing the serialized hash field does not affect the loaded canonical hash.
- Loaded states use the recomputed hash.
- Hash recomputation uses the same rule as normal engine state hashing.

**Dependencies:**  
REQ-HASH-001.

---

#### REQ-JSON-003 — Validation on load

**Statement:**  
State loading **MUST** validate the loaded state before returning it to callers.

**Rationale:**  
JSON is a trust boundary. Invalid or corrupted state should fail early and explicitly.

**Acceptance Criteria:**

- Invalid status values are rejected.
- Invalid mode values are rejected.
- Invalid scoring values are rejected.
- Board masks with invalid padding bits are rejected.
- Overlapping color masks are rejected.
- Duo states with invalid consistency are rejected.
- Rejections use structured input or engine errors.

**Dependencies:**  
REQ-ERR-003.

---

## 6. Quality Attributes

#### REQ-QA-PERF-001 — Mask-algebra hot paths

**Statement:**  
Move generation and placement validation **SHOULD** primarily use board mask algebra instead of per-cell scans, except when individual cells must be emitted to callers.

**Rationale:**  
Move generation is a likely hot path for UI hints, bots, and future search algorithms.

**Acceptance Criteria:**

- Placement validation uses board mask operations for overlap, bounds, and contact checks.
- Move generation uses mask operations for target and forbidden cells.
- Per-cell iteration is limited to output conversion or small shape iteration.
- Future benchmarks should compare against a brute-force reference approach.

**Dependencies:**  
REQ-CORE-002, REQ-MOVE-003.

---

#### REQ-QA-MAINT-001 — Core coverage threshold

**Statement:**  
The pure engine implementation **SHALL** maintain a minimum line coverage threshold.

**Rationale:**  
The engine is the source of truth for rules. Regressions in this layer are costly and hard to diagnose from the UI or backend.

**Acceptance Criteria:**

- The CI pipeline runs coverage for the core engine.
- The CI pipeline fails if the configured threshold is not met.
- Coverage is treated as a release gate for the engine package.

**Dependencies:**  
None.

---

#### REQ-QA-MAINT-002 — Lint discipline

**Statement:**  
The Rust workspace **SHALL** build cleanly under strict formatting and linting rules.

**Rationale:**  
Strict linting catches avoidable code quality issues and keeps generated or AI-assisted code reviewable.

**Acceptance Criteria:**

- Rust formatting passes in CI.
- Rust linting passes in CI.
- Warnings are not silently ignored in release checks.
- Unsafe code remains forbidden.

**Dependencies:**  
REQ-CORE-005.

---

#### REQ-QA-TEST-001 — Cross-language test coverage

**Statement:**  
Both Rust core tests and Python binding tests **SHALL** be runnable through the project test commands and CI pipeline.

**Rationale:**  
The Rust core and Python binding are both part of the engine deliverable. They must be tested together.

**Acceptance Criteria:**

- Rust core tests pass.
- Python binding tests pass.
- The Python extension can be built before binding tests run.
- The aggregate project check runs formatting, linting, coverage, and binding tests.

**Dependencies:**  
REQ-QA-MAINT-001, REQ-QA-MAINT-002.

---

## 7. Verification Strategy

Every functional requirement should be verified by at least one of the following:

- Rust unit tests for pure domain behavior
- Rust property tests where useful for invariants
- Python integration tests for the binding layer
- JSON round-trip tests
- CI checks for formatting, linting, coverage, and cross-language integration
- Manual code review for quality goals that are not fully testable yet

The Python integration tests should cover:

- placement success and placement rule violations
- command value behavior
- game configuration and initialization
- Classic and Duo setup
- board, piece, and inventory views
- move generation and pass behavior
- shared-color ownership in three-player mode
- error hierarchy and structured error messages
- scoring gates
- JSON serialization and deserialization
- public typing and binding contract

A requirement is considered met when:

1. its acceptance criteria are implemented,
2. the related tests pass,
3. the aggregate CI check passes,
4. and any remaining limitations are documented as open items.

---

## 8. Open Items

- **Engine benchmarks:** A benchmark suite for move generation and hash transitions should be added once AI/search work becomes performance-sensitive.
- **Replay format:** Deterministic hashing makes replay support possible, but the replay schema is not specified yet.
- **Three-player Advanced scoring coverage:** Advanced scoring exists for all modes, but specific three-player Advanced scoring tests should be expanded when the backend exposes this flow.
- **Direct finished-command rejection tests:** Finished-state command rejection should have direct tests for both placement and pass commands if not already covered.
- **Exact Rust property-test traceability:** Any requirement that claims property-test coverage should link to an explicit existing test or be marked as planned coverage.