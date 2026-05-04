# Portfolio_Tobias_Friedrich.md

> **Individual Student Portfolio**  
> *~4-5 page report documenting your contributions, guideline applications, and counterexamples.*

---

## Student Information

**Student Name:** `Tobias Friedrich`  
**Team Name:** `Design`  
**Project:** Blokus Game Engine (Classic + Duo)  

---

## 1. Owned Package Contributions

### Package Name: Pipeline Setup and CI Documentation

**Description:**  
I setup the GitHub CI pipeline. The pipeline runs checks for the Flutter frontend, FastAPI backend, Rust engine, and Python FFI bindings. It is used on pull requests, pushes to `main`, and manual runs.

**Responsibilities:**  
- Set up the GitHub Actions workflow for frontend, backend, and engine validation.
- Add formatting, linting, type checking, test, and coverage steps.
- Make sure the Rust Python binding is built during backend and engine checks.

**Evidence Links:**
- **CI Pipeline:** [`/.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
- **Engine Makefile:** [`engine/Makefile`](../../engine/Makefile)
- **FastAPI Makefile:** [`backend/Makefile`](../../backend/Makefile)
- **Flutter Makefile:** [`frontend/Makefile`](../../frontend/Makefile)

**Key Contributions:**
- Added separate CI jobs for frontend, backend, and engine.
- Added format, lint, typecheck, test, and coverage checks.

---

### Package Name: `blocus-core`

**Description:**  
I owned the Rust core engine package. This package contains the main game logic for Classic Blokus and Blokus Duo. It handles board state, player/color setup, piece definitions, move validation, move generation, turn tracking, passing, scoring, hashing, and state validation. It is designed as a pure Rust domain layer without direct dependencies on FastAPI, Redis, Flutter, or Python runtime logic.

**Responsibilities:**  
- Implement the core domain model for Blokus.
- Support Classic and Duo game modes.
- Enforce placement rules and opening rules.
- Generate legal moves.
- Track turns, passed colors, finished games, and score state.
- Keep the state compact and deterministic.
- Provide stable Rust APIs for the Python binding layer.
- Add tests and invariants for correctness and coverage.

**Evidence Links:**
- **Core Engine Crate:** [`engine/crates/blocus-core/`](../../engine/crates/blocus-core/)
- **Engine Facade:** [`engine/crates/blocus-core/src/engine/mod.rs`](../../engine/crates/blocus-core/src/engine/mod.rs)
- **Board Logic:** [`engine/crates/blocus-core/src/board/`](../../engine/crates/blocus-core/src/board/)
- **Game Config:** [`engine/crates/blocus-core/src/config/mod.rs`](../../engine/crates/blocus-core/src/config/mod.rs)
- **Move Generation:** [`engine/crates/blocus-core/src/movegen/mod.rs`](../../engine/crates/blocus-core/src/movegen/mod.rs)
- **Placement Rules:** [`engine/crates/blocus-core/src/rules/placement.rs`](../../engine/crates/blocus-core/src/rules/placement.rs)
- **Scoring:** [`engine/crates/blocus-core/src/scoring/mod.rs`](../../engine/crates/blocus-core/src/scoring/mod.rs)
- **Hashing:** [`engine/crates/blocus-core/src/hash/mod.rs`](../../engine/crates/blocus-core/src/hash/mod.rs)
- **Transposition Table:** [`engine/crates/blocus-core/src/transposition.rs`](../../engine/crates/blocus-core/src/transposition.rs)
- **Tests** [`engine/crates/blocus-core/tests/`](../../engine/crates/blocus-core/tests/)

**Key Contributions:**
- Implemented the core `BlocusEngine` facade with methods for:
  - game initialization
  - command application
  - legal move generation
  - move existence checks
  - final scoring
- Added mode-aware configuration through:
  - `GameMode`
  - `GameConfig`
  - `PlayerSlots`
  - `TurnOrder`
  - `Ruleset`
  - `BoardGeometry`
- Added support for Classic and Duo:
  - Classic: 20x20 board, blue/yellow/red/green colors, corner starts
  - Duo: 14x14 board, black/white colors, start points at `(4, 4)` and `(9, 9)`
- Implemented placement validation:
  - rejects overlap
  - rejects out-of-bounds placements
  - enforces same-color corner contact
  - rejects same-color edge contact
  - checks first move start positions
- Implemented a legal move iterator using board masks.
- Added compact board representation with padded-row bit indexing.
- Added inventory tracking for all 21 Blokus pieces.
- Added advanced scoring with completion and monomino-last bonuses.
- Added deterministic position hashing for state verification and future AI/search usage.
- Added a transposition table primitive for later search-based AI work.
- Added structural state validation to reject corrupted board masks, invalid turn state, inactive colors, and invalid Duo state.

---

### Package Name: `blocus-python`

**Description:**  
Furthermore, I owned the Python binding package for the Rust core engine. This package exposes the Rust engine to Python using PyO3. It gives the backend a Python-friendly API while still executing game logic in Rust. It also handles conversion between Python objects and Rust domain types, structured errors, JSON state round trips, and public API tests.
In a next sprint, we can develop a similar package to provide dart / ... bindings for different languages.

**Responsibilities:**  
- Expose the Rust core engine to Python.
- Convert Python input types into validated Rust domain types.
- Provide Python classes for game config, commands, state, results, pieces, and enums.
- Map Rust errors into structured Python exceptions.
- Support JSON serialization and deserialization of game state. (As requested for this Team Project)
- Add Python tests for the binding contract.
- Keep the Python API stable enough for backend usage.
- Provide pyi bindings for python linters

**Evidence Links:**
- **Python Binding Crate:** [`engine/crates/blocus-python/`](../../engine/crates/blocus-python/)
- **PyO3 Module Entry:** [`engine/crates/blocus-python/src/lib.rs`](../../engine/crates/blocus-python/src/lib.rs)
- **Engine Binding:** [`engine/crates/blocus-python/src/engine.rs`](../../engine/crates/blocus-python/src/engine.rs)
- **Command Binding:** [`engine/crates/blocus-python/src/command.rs`](../../engine/crates/blocus-python/src/command.rs)
- **Config Binding:** [`engine/crates/blocus-python/src/config.rs`](../../engine/crates/blocus-python/src/config.rs)
- **State Binding:** [`engine/crates/blocus-python/src/state.rs`](../../engine/crates/blocus-python/src/state.rs)
- **Result Binding:** [`engine/crates/blocus-python/src/result.rs`](../../engine/crates/blocus-python/src/result.rs)
- **Piece Binding:** [`engine/crates/blocus-python/src/pieces.rs`](../../engine/crates/blocus-python/src/pieces.rs)
- **Error Mapping:** [`engine/crates/blocus-python/src/errors.rs`](../../engine/crates/blocus-python/src/errors.rs)
- **Python Tests:** [`engine/crates/blocus-python/python_tests/`](../../engine/crates/blocus-python/python_tests/)
- **PYI Python Types** [`backend/src/blocus_engine.pyi`](../../backend/src/blocus_engine.pyi)

**Key Contributions:**
- Added the Python `BlocusEngine` wrapper with methods for:
  - `initialize_game`
  - `apply`
  - `get_valid_moves`
  - `get_valid_moves_for_piece`
  - `has_any_valid_move`
  - `has_any_valid_move_for_piece`
  - `score_game`
- Added Python command objects:
  - `PlaceCommand`
  - `PassCommand`
- Added Python config objects:
  - `GameConfig`
  - `GameMode`
  - `PlayerSlots`
  - `SharedColorTurn`
- Added Python state APIs:
  - `board_matrix`
  - `cell`
  - `occupied_cells`
  - `board_counts`
  - `used_piece_ids`
  - `available_piece_ids`
  - `inventory_summary`
  - `to_json`
  - `from_json`
- Added Python wrappers for pieces and orientations.
- Added stable enum-like classes for:
  - `PlayerColor`
  - `GameStatus`
  - `ScoringMode`
  - `GameMode`
  - `DomainEventKind`
  - `DomainResponseKind`
- Added structured Python errors:
  - `BlocusError`
  - `InputError`
  - `RuleViolationError`
  - `EngineError`
- Added JSON round-trip support with hash recomputation and state validation.
- Added Duo-specific Python API support:
  - `GameMode.DUO`
  - `PlayerColor.BLACK`
  - `PlayerColor.WHITE`
  - `GameConfig.duo(...)`
  - 14x14 board output
  - Duo opening rules
  - Duo JSON state round trips
- Added Python tests covering commands, enums, config, state JSON, board/piece APIs, Duo mode, edge cases, errors, passing, and scoring.

---

## 2. Guideline Applications

> **Note:** Document at least 3 applications of guidelines from other teams' guideline packages. For each, describe the guideline, how you applied it, and the outcome.

### Application 1: `[Guideline Name]` from `[Topic]` Team

**Guideline Description:**  
Briefly describe the guideline you applied.

**Context:**  
What task or feature were you working on when you applied this guideline?

**Application Process:**  
1. `[Step 1]`
2. `[Step 2]`
3. `[Step 3]`

**Outcome:**  
- **What worked:** `[Description]`
- **What didn't work:** `[Description]`
- **Evidence:** `[Link to prompt, code, tests, or documentation]`

**Reflection:**  
What did you learn from applying this guideline? Would you use it again in a similar context?

---

### Application 2: `[Guideline Name]` from `[Topic]` Team

**Guideline Description:**  
Briefly describe the guideline you applied.

**Context:**  
What task or feature were you working on when you applied this guideline?

**Application Process:**  
1. `[Step 1]`
2. `[Step 2]`
3. `[Step 3]`

**Outcome:**  
- **What worked:** `[Description]`
- **What didn't work:** `[Description]`
- **Evidence:** `[Link to prompt, code, tests, or documentation]`

**Reflection:**  
What did you learn from applying this guideline? Would you use it again in a similar context?

---

### Application 3: `[Guideline Name]` from `[Topic]` Team

**Guideline Description:**  
Briefly describe the guideline you applied.

**Context:**  
What task or feature were you working on when you applied this guideline?

**Application Process:**  
1. `[Step 1]`
2. `[Step 2]`
3. `[Step 3]`

**Outcome:**  
- **What worked:** `[Description]`
- **What didn't work:** `[Description]`
- **Evidence:** `[Link to prompt, code, tests, or documentation]`

**Reflection:**  
What did you learn from applying this guideline? Would you use it again in a similar context?

---

## 3. Counterexamples

> **Note:** Document at least 3 reproducible counterexamples where guidelines failed or produced suboptimal results. For each, include the failure, diagnosis, and refinement.

### Counterexample 1: `[Title]`

**Failure Description:**  
Describe the failure or suboptimal result. What guideline was applied? What was the expected outcome? What actually happened?

**Diagnosis:**  
- **Root Cause:** `[Description]`
- **Why the Guideline Failed:** `[Description]`
- **Boundary Condition:** `[Description of when the guideline fails]`

**Refinement:**  
- **Updated Guideline:** `[Description of the refined guideline]`
- **How It Was Tested (evaluated):** `[Description of testing]`
- **Evidence:** `[Link to code, tests, or documentation]`

**Prompt/Context Used:**  
```
[Paste the prompt or context you used with the AI tool]
```

**AI Output:**  
```
[Paste the AI output that failed or was suboptimal]
```

---

### Counterexample 2: `[Title]`

**Failure Description:**  
Describe the failure or suboptimal result.

**Diagnosis:**  
- **Root Cause:** `[Description]`
- **Why the Guideline Failed:** `[Description]`
- **Boundary Condition:** `[Description of when the guideline fails]`

**Refinement:**  
- **Updated Guideline:** `[Description of the refined guideline]`
- **How It Was Tested (evaluated):** `[Description of testing]`
- **Evidence:** `[Link to code, tests, or documentation]`

**Prompt/Context Used:**  
```
[Paste the prompt or context you used with the AI tool]
```

**AI Output:**  
```
[Paste the AI output that failed or was suboptimal]
```

---

### Counterexample 3: `[Title]`

**Failure Description:**  
Describe the failure or suboptimal result.

**Diagnosis:**  
- **Root Cause:** `[Description]`
- **Why the Guideline Failed:** `[Description]`
- **Boundary Condition:** `[Description of when the guideline fails]`

**Refinement:**  
- **Updated Guideline:** `[Description of the refined guideline]`
- **How It Was Tested (evaluated):** `[Description of testing]`
- **Evidence:** `[Link to code, tests, or documentation]`

**Prompt/Context Used:**  
```
[Paste the prompt or context you used with the AI tool]
```

**AI Output:**  
```
[Paste the AI output that failed or was suboptimal]
```

---

## 4. AI Usage Disclosure

### Tools and Models Used

| Tool/Model | Usage | Validation Method |
|------------|-------|-------------------|
| `[e.g., GitHub Copilot]` | `[e.g., Code generation, suggestions]` | `[e.g., Unit tests, peer review]` |
| `[e.g., GPT-4]` | `[e.g., Requirements, documentation]` | `[e.g., Manual verification]` |
| `[e.g., Claude 3.5 Sonnet]` | `[e.g., Test generation]` | `[e.g., Test execution]` |

### Evaluation Methods

Describe how you evaluated AI-generated outputs (below are examples for your guidance):

1. **Correctness Testing:** `[Description]`
2. **Code Review:** `[Description]`
3. **Unit Tests:** `[Description]`
4. **Integration Tests:** `[Description]`
5. **Performance Testing:** `[Description]` (if applicable)

### Time Investment

Approximately how much time did you spend on:
- AI prompting and refinement: `[X] hours`
- Reviewing AI outputs: `[X] hours`
- Testing and validation: `[X] hours`
- Documentation: `[X] hours`

---

## 5. Reflections

> **Note:** Use this as your guidance

### What You Learned

- `[Lesson 1]`
- `[Lesson 2]`
- `[Lesson 3]`

### Skills Developed

- `[Skill 1]`
- `[Skill 2]`
- `[Skill 3]`

### Future Improvements

If you could do this project again, what would you do differently?

- `[Improvement 1]`
- `[Improvement 2]`
- `[Improvement 3]`

---

## Instructions for Use

1. **Replace all `[...]` placeholders** with your specific content
2. **Document at least 3 guideline applications** with evidence
3. **Document at least 3 counterexamples** with proper analysis
4. **Be specific about AI tools used** and how outputs were validated
5. **Keep it concise** (4-5 pages max)
6. **Submit as `Portfolio_<StudentName>.md`** (replace `<StudentName>` with your actual name) in your project repository

---

*Template version: 1.0 | Last updated: 24 February 2026*
