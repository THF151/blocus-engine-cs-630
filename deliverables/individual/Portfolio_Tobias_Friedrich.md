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

### Application 1: `
`Detailed Personas (G2)` + `Proactive Ambiguity Clarification` (G3) from Requirements Team

**Guideline Description:**  
A combined application of two guidelines from the Requirements team. G2 says to give the LLM a rich, structured persona (e.g. as JSON) so the elicitation reflects the concerns that persona would care about. G3 says the LLM should flag unclear statements *before* generating requirements, ask short clarification questions, and then embed the answers back into the requirement text. I also kept **G1 (Human-in-the-Loop)** active in the background: after each pass, I reviewed the output and gave short feedback to steer the next iteration.

**Context:**  
The first elicitation pass for the Rust `blocus` engine. The engine had to cover Classic Blokus, and later — through a change request — Blokus Duo. It also had to expose a clean FFI surface usable from Python (`blocus-python`). The output target was a set of user stories using MUST / SHALL / SHOULD modal verbs, with acceptance criteria, so that we could turn them directly into engine work items and plan our implementation.

**Application Process:**
1. Defined a persona JSON for a senior CS professor focused on maintainable code and optimization. I pinned it as the first turn so the LLM prioritized clean domain modeling, validation, and performance-aware data structures instead of generic "good software" advice.
2. Provided the official Blokus Classic rules as the main input, plus the constraint that the system must expose a Python-usable FFI API alongside with the team projects relevant slides. Asked the LLM to produce user stories in MUST / SHALL / SHOULD form.
3. Applied G3: told the LLM that, before writing any requirement, it had to flag places where the rules text was unclear (for example: scoring bonuses, behaviour on a forced pass, tie-breaking) and ask short clarification questions. Once a question was answered, the answer had to be embedded into the matching requirement.
4. After the first pass, applied G1: I reviewed the draft, noticed that the stories were well-formed but missed acceptance criteria and dependency links, and gave short human feedback asking for an explicit output schema (acceptance criteria, inter-story dependencies, language target — Rust core + PyO3 binding). The next pass produced directly usable output.
5. Issued a change request to extend the same elicitation to Blokus Duo (14×14 board, two colors, fixed start points), reusing the persona and the clarification protocol so the Duo stories sat in parallel with the Classic ones.

**Outcome:**
- **What worked:** The persona kept the LLM focused on the right concerns — state representation, placement invariants, FFI shape — instead of UI-flavored stories. G3 caught real ambiguities in the rules text (what happens on a tie, starting corner coordinates in duo rule text not clearly mentioned, ...) and forced explicit Q&A turns; The Duo change request reused the same persona context cleanly and produced parallel stories without restarting from scratch.
- **What didn't work (initially):** The first run produced clean stories but without acceptance criteria or dependency links, so they weren't directly actionable. This wasn't a problem with G2 or G3 — it was a missing output schema in my prompt. A short human review (G1) and one round of feedback fixed it: the persona and clarification core stayed the same, only the format spec was added.
- **Evidence:**
  - Requirements document produced by this elicitation: [`deliverables/individual/evidence/requirements-engine.md`](../../deliverables/individual/evidence/requirements-engine.md)

**Reflection:**  
Of course assigning a role is like the first rule in Prompt Engeneering 101, yet it seems to work out. The takeaway on the schema is worth keeping: neither G2 nor G3 prescribes an output format, and without one the LLM will fall back to loose prose. I would apply this combination again for any rule-driven domain (board games, regulatory specs, protocol implementations) where the source text has known soft spots. For more open-ended or creative domains I would weaken the persona and probably drop G3, since "ambiguity" there is often the point.

---

### Application 2: `UML Specification (Sequence Diagrams)` from `Design` Team

**Guideline Description:**  
Guideline 3 is the four-stage UML workflow: prepare inputs → core prompting principles → diagram-type-specific guidance → validation. For sequence diagrams specifically, the relevant elements are: explicit instructions on `loop`/`alt`/`opt` fragments (not letting the LLM scatter or over-nest them), manual verification of the initiating actor, inline `note right` traceability with REQ IDs at every major message or alternative block, and storing the result as text-based diagrams-as-code in version control rather than as static images.

**Context:**  
We needed sequence diagrams to ground the actual design — specifically the two highest-traffic flows that cross the Python-Rust boundary: applying a place command (`engine.apply(state, PlaceCommand)`), and legal-move generation (`engine.get_valid_moves(state, player, color)` ultimately driving `LegalMoveIter`). We needed a way to plan the design fast and discuss possible optimization strategies to avoid expensive runtime mistakes and to enable proper AI Training in a later sprint. Of course these can be discussed in chat, but UML Diagrams are much faster as long as the LLM can produce them syntactically and semantically correct.

**Application Process:**
1. Started a fresh LLM session and fed it the serialized engine prototype codebase. Alongside we fed in the original Requirement document and a basic ADR. This is the Stage-1 "architecture documentation as primary input" step.
2. Asked for two sequence diagrams (one per flow), explicitly required Mermaid as the output notation, and listed the participants up front to keep them under the recommended cap. Required `note right` blocks tagged with REQ IDs (`REQ-FFI-001`, `REQ-RULE-001…008`, `REQ-MOVE-001`, …) at every validation step — Stage 2 traceability requirement.
3. Applied the sequence-specific guidance from Stage 3: explicitly told the LLM where loops were genuinely cyclic and where alternatives were branching. Without that, the first draft tended to either flatten the iteration into a straight call sequence or over-nest decisions.
4. Manually verified the initiating actor on both diagrams (Stage 3, sequence guidance #2). Both flows do start at the Python caller — there are no event-driven background paths in the engine — so no correction was needed here, but the check was deliberate rather than assumed.
5. Validated by rendering the Mermaid in a live preview and reviewing the result against the actual code. Fixed minor syntax issues from the first draft, then iterated once on the move-gen flow because the first version expressed the iteration semantics correctly but described an inefficient walk that didn't match the lazy cursor in `LegalMoveIter`.

**Outcome:**
- **What worked:** Planning the design visually was significantly faster than reading through code-snippets or large adrs line by line. It allowed for a very fast iteration discussing different kinds of optimization methods. By using a separate chat, we were able to come up with even more room for improvement. The traceability notes also paid off: every check in rule validation is now explicitly tied to a REQ ID, which makes it easy to spot later if a check is missing. The `loop` / `alt` / `opt` instructions kept the diagrams structurally honest. To ensure feasibility of complex algorithms, we made it propose short code snippets alongside the diagram.
- **What didn't work (initially):** Two issues, both minor and both fixable in one round. First, Mermaid syntax: the first draft had inconsistent activation/deactivation pairs (more `+` than `-`) and used `\` for set difference in a note, which the parser didn't like. This was solved very quickly. Second, and more interesting, the first move-gen draft proposed a structurally correct but algorithmically inefficient loop for move generation stressing the necessity for a human or specifically prompted "Efficiency-Expert"-LLM oversight.
- **Evidence:**
  - Sequence diagrams (`apply(PlaceCommand)` + `get_valid_moves` / `LegalMoveIter`): [`deliverables/individual/evidence/uml-engine.md`](../../deliverables/individual/evidence/uml-engine.md)

**Reflection:**  
The biggest takeaway is that visual planning beats code or adr reading — by a lot — for understanding algorithmic control flow. Once the diagrams were drafted, the engine's behavior fit on one page each, and the few real surprises were exactly the things worth discussing. The Stage-3 sequence-specific guidance pulled most of the weight here: explicit `loop`/`alt`/`opt` framing prevented both flattening and over-nesting, and the REQ-ID traceability turned the diagrams from illustrations into reviewable specifications. I would apply this again for any flow that crosses an API or FFI boundary, where understanding *who* calls *whom* and *which checks happen at which layer* - not only for planning but also for documentation purposes. I would also keep the same rule of thumb — let the LLM draft, then push back on anything that looks structurally plausible but doesn't match best practices.

---

### Application 3: `Iterative Remediation and Self-Correction Loops` from `Coding` Team

**Guideline Description:**  
Coding Guideline 3 recommends a structured Plan-Execute-Review loop for AI-assisted coding. The first output should be treated as a draft, not as final code. Failed test output, formatter errors, linting errors, and review findings should be fed back into the model so it can remediate the implementation. The guideline also recommends using a reviewer-style pass to catch silent hallucinations such as incomplete behavior, performance problems, or security issues that may not appear as syntax errors.

**Context:**  
I applied this guideline when extending the Rust `blocus-core` engine and the `blocus-python` binding from Classic Blokus to Blokus Duo. The Duo variant affected several parts of the system: board geometry, active colors, opening targets, configuration validation, scoring restrictions, JSON round trips, Python-facing APIs, and tests. Because the change crossed both Rust and Python boundaries, a one-shot implementation would have been risky.

**Application Process:**
1. After working out the initial requirements and required behavioral changes, I started a Codex task specifically for the Duo variant.
2. I first asked Codex to plan the implementation before writing code. The plan decomposed the Duo extension into smaller changes such as Duo board geometry, Duo color support, fixed starting points, scoring-mode restrictions, state validation, Python API exposure, and test coverage.
3. I then let Codex execute the implementation incrementally. I explicitly instructed it to keep the repository in a working state and to verify changes through the existing project pipeline, especially `make fmt` and `make check`.
4. I required Codex to generate tests alongside the implementation, preferably through the public/friendly API first. This made the expected Duo behavior executable before relying only on internal implementation checks.
5. Whenever validation exposed a failure or missing behavior, the error output or review finding was fed back into Codex for remediation before moving to the next step. This followed the guideline’s Plan-Execute-Review loop rather than accepting the first generated patch.
6. After Codex produced the implementation, I manually reviewed the result against the Duo requirements and existing Classic behavior. This acted as the reviewer pass for silent issues: the code had to preserve Classic mode, expose Duo cleanly through Python, and avoid implementing only the happy path.

**Outcome:**
- **What worked:** This guideline matched the Codex workflow very well. Codex was able to plan, implement, run or respond to validation steps, and revise the implementation until the codebase passed the existing checks. The result was a working Duo implementation in about 20 minutes, including tests and Python-facing support. The existing pipeline made the remediation loop concrete: failures were not discussed abstractly but tied to formatter, lint, test, and coverage feedback.
- **What didn't work:** The main limitation was not the quality of the generated code, but the practical availability of the tool. In recent weeks, Codex and Claude Code usage limits became much more restrictive for our workflow, effectively allowing only about one comprehensive task every five hours. That makes this approach hard to rely on for larger continuous workflows, even though the individual task quality was good.
- **Evidence:**
  - Codex-generated Duo implementation commit: [`https://github.com/THF151/blocus-engine-cs-630/commit/58ec04ed5b595249a3fb9920481a0fea8e2aa656`](https://github.com/THF151/blocus-engine-cs-630/commit/58ec04ed5b595249a3fb9920481a0fea8e2aa656)
  - The commit shows the planned, implemented, tested, and remediated Duo extension across the Rust core, Python binding, and test suite.
  - The relevant changes include Duo board geometry, Duo colors, Duo opening rules, scoring-mode validation, JSON round trips, Python API exposure, and additional Rust/Python tests.

**Reflection:**  
This guideline fits the Duo implementation better than a pure decomposition guideline because the decisive part was not only splitting the work, but repeatedly validating and repairing the generated code. Codex already incorporates much of this workflow: it plans, edits, observes failures, and revises. However, the guideline was still necessary because we had to define the verification contract: use the existing pipeline, keep the codebase green, generate friendly API tests first, target high coverage, and preserve Classic behavior. My main takeaway is that agentic coding is most useful when the repository already has a strong validation pipeline. Without that pipeline, the model could still produce plausible code, but there would be no reliable remediation signal.

---

## 3. Counterexamples

> **Note:** Document at least 3 reproducible counterexamples where guidelines failed or produced suboptimal results. For each, include the failure, diagnosis, and refinement.

---

### Counterexample 1: `Atomic Task Decomposition (G4)` from `Coding` Team

**Failure Description:**
Coding G4 recommends breaking complex requirements into atomic, testable units (functions/modules) and prompting for them individually with a small chain-of-reasoning. The guideline's example — "Instead of *Build a full data ingestion pipeline*, first prompt for *A function to parse the raw headers*" — implicitly assumes the function is self-contained. I applied this exact pattern in two settings: during the early prototyping of the engine, and later when asking for specific methods on `blocus-core` (placement legality, scoring helpers, FFI conversion). The expected outcome was a clean, focused function that I could drop into the existing module. What I actually got — repeatedly — was code that looked correct in isolation but did not fit the codebase: it reinvented utilities that already existed (e.g. bitmask intersection), hallucinated plausible-but-wrong type signatures for `Board`, `Piece`, and `Position`, ignored the `BlocusError` mapping convention used everywhere else in the crate. None of it passed human review and none of it was committed, but each round still cost time to read, diagnose, and discard.

**Diagnosis:**
- **Root Cause:** G4 prescribes the *granularity* of the prompt (one function, one module) but says nothing about what surrounding context must travel with each atomic prompt. Once a request is scoped to a single function, the LLM has no signal about the public interfaces, error conventions, or precomputed utilities of the surrounding module. It fills the gap with invented but plausible structures that fit the prompt locally and clash with the codebase globally.
- **Why the Guideline Failed:** G4 is in unacknowledged tension with Coding G1 (Context-Aware Grounding via AGENTS.md). G1's premise is that >70% of real-world functions depend on project-specific entities and therefore need repository context; G4's prompt-shape pulls in the opposite direction by isolating one function at a time. The package treats the two as independent, but in practice applying G4 without an explicit context-carry mechanism produces exactly the hallucinated-dependency failures G1 is meant to prevent.
- **Boundary Condition:** The failure is reproducible whenever (a) the surrounding module has non-obvious internal utilities (in our case for example the bitmask board representation, (b) the crate maintains a strict dependency boundary that isn't visible from the function signature alone, or (c) error handling follows a specific convention rather than a language default (`Result<(), BlocusError>` with structured variants. For straightforward standard-library tasks the guideline works; for codebase-shaped tasks it does not.

**Refinement:**
- **Updated Guideline:** Treat G4 as inseparable from G1. Every atomic prompt must carry, in addition to the function specification: (i) the AGENTS.md or equivalent project conventions, (ii) the public interface and relevant trait definitions of the surrounding module, and (iii) any utility helpers the function is expected to use. Concretely: do not prompt for a function in isolation. Prompt for a function plus a serialized slice of the repository it has to live inside.
- **How It Was Tested (evaluated):** I built a serialization script that walks the repository and emits a single markdown bundle containing the public API of all modules, the relevant trait/type definitions. The bundle is sized to fit comfortably inside ChatGPT's ~100k-token context window. I then re-ran the same atomic prompt on `is_legal_placement` and several other targets, this time with the serialized bundle prepended. The same prompt that previously hallucinated several redundancies and assumptions now returned to the bitmask representation, called the existing geometry helpers, and returned `Result<(), BlocusError>` with the correct variants. The improvement was consistent across the targets I tried, not a one-off.
- **Evidence:**
  - Repository serialization script: [`engine/serialize.py`](../../engine/serialize.py) 

**Prompt/Context Used:**
```
Implement a Rust function for the Blokus engine:

fn is_legal_placement(
    board: &Board,
    piece: &Piece,
    position: Position,
    player: Player,
) -> bool

The function checks that the piece, placed at `position` for `player`,
fits inside the board, does not overlap any existing piece, does not
edge-touch a same-color piece, and corner-touches at least one
same-color piece (first move excepted: must cover the player's start
square instead). Use chain-of-thought to reason through the cases
before writing the code, and include unit tests.
```

**AI Output:**
```rust
pub fn is_legal_placement(
    board: &Board,
    piece: &Piece,
    position: Position,
    player: Player,
) -> bool {
    let mut absolute_cells = Vec::with_capacity(piece.cells.len());

    for relative in &piece.cells {
        let absolute = Position {
            row: position.row + relative.row,
            col: position.col + relative.col,
        };

        // 1. Piece must fit entirely inside the board.
        if !Board::in_bounds(absolute) {
            return false;
        }

        // 2. Piece must not overlap any existing piece.
        if board.get(absolute).is_some() {
            return false;
        }

        absolute_cells.push(absolute);
    }

    let is_first_move = !board.has_player_piece(player);

    if is_first_move {
        let start_square = player.start_square();

        // First move must cover the player's starting square.
        return absolute_cells.contains(&start_square);
    }

    let edge_offsets = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
    ];

    let corner_offsets = [
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
    ];

    let mut has_same_color_corner_contact = false;

    for cell in &absolute_cells {
        // 3. Cannot edge-touch a same-color piece.
        for (dr, dc) in edge_offsets {
            let neighbor = Position {
                row: cell.row + dr,
                col: cell.col + dc,
            };

            if Board::in_bounds(neighbor) && board.get(neighbor) == Some(player) {
                return false;
            }
        }

        // 4. Must corner-touch at least one same-color piece.
        for (dr, dc) in corner_offsets {
            let neighbor = Position {
                row: cell.row + dr,
                col: cell.col + dc,
            };

            if Board::in_bounds(neighbor) && board.get(neighbor) == Some(player) {
                has_same_color_corner_contact = true;
            }
        }
    }

    has_same_color_corner_contact
}
```
*Failure modes visible in this output:* (1) `Board`, `Piece`, `Position`, `Player` are all hallucinated — none of these signatures exist in `blocus-core`. This is not surprising without proper context engeneering. 

---

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

| Tool/Model            | Usage                                                                                    | Validation Method                                              |
|-----------------------|------------------------------------------------------------------------------------------|----------------------------------------------------------------|
| `GPT 5.5 Chat`        | Code generation, planning, discussion, UML generation                                    | Automated testing pipeline, human review                       |
| `Opus 4.7`            | Code reviews on PRs, serialized codebase to spot logical errors and optimization options | Human review                                                   |
| `Codex / Claude Code` | Code generation (limited by tight usage limits)                                          | Automated testing pipeline, human review of tests and codebase |

### Evaluation Methods

Describe how you evaluated AI-generated outputs:

1. **Correctness Testing:** Automated testing pipeline checks against domain logic.
2. **Code Review:** Human review of both the generated test suite and the actual codebase.
3. **Unit Tests:** Automated test runs via `make check` on PRs and local validations.
4. **Integration Tests:** Python FFI layer tested against Rust code continuously.
5. **Performance Testing:** A simulation script was created to measure runtime and memory space performance between different versions.

### Time Investment

Approximately how much time did you spend on: (I can only recommend to put this at the beginning of the document for the next classes)
- AI prompting and refinement: `15 hours`
- Reviewing AI outputs: `30 hours`
- Testing and validation: `30 hours`
- Documentation: `30 hours`

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
