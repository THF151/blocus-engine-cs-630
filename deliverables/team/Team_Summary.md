# Team_Summary.md

> **Team Project Summary**  
> *1-2 page overview of your team's project, AI usage, and key results.*

---

## Team Information

**Team Name:** `Design`  
**Project:** Blokus Game Engine (Classic + Duo)  
**Team Members:** Aleksander Kasak, Stephan Herbert, Tobias Friedrich

---

## Project Scope & Architecture

### Overview

The project implements a full-stack Blokus Game Engine encompassing both Classic and Duo variants. The tech stack has a Rust core engine for game logic and move validation, integrated via FFI into a Python (FastAPI) backend service. The backend architecture is cloud-native and orchestrates multiplayer connections via HTTP/WebSockets and utilizes Redis for state management. Players interact through a Flutter frontend that supports iOS/Android/Web/Desktop

### Architecture Diagram



### Key Components

1. **Rust Core Engine:** Handles core rules, board dimensions, legal placement evaluations, turn tracking, and scoring calculations
2. **Backend Service Layer:** Manages request endpoints, WebSocket connections, concurrency, and interfaces directly with the Rust engine
3. **State Management:** Redis instance used for horizontal scalability and managing active lobby/game states
4. **Flutter Frontend:** Provides the interactive user interface, rendering lobbies, and facilitating real-time drag-and-drop gameplay syncing with the backend

---

## AI Tools Used

### High-Level Overview

We used the latest available models for each phase and validated their output through our automated pipeline plus human (and cross-model) review. The table consolidates the per-member usage documented in the individual portfolios.

| Phase                          | AI Tool/Model                                                        | Usage                                                                                                                                          | Validation Method                                                          |
|--------------------------------|----------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------|
| Requirements                   | GPT 5.5                                                             | Requirements elicitation with detailed personas and proactive ambiguity clarification                                                          | Manual review, LLM-as-judge                                                |
| Design                         | GPT 5.5                                                             | ADR generation and UML sequence diagrams for the Python ↔ Rust flows                                                                           | Manual review, live diagram rendering against the code                     |
| Code Gen & Testing & Debugging | Codex (GPT-5.x), Claude Code (Opus 4.7), GPT 5.5 Chat, GitHub Copilot | Rust engine, Python backend, and Flutter UI; test suites; planning and `PROTOCOL.md` drafting (Copilot only for occasional inline completion)   | `make check` (format, lint, strict typing, tests, coverage) + human review |
| Code Review                    | Opus 4.7, cross-instance Codex ↔ Claude Code                         | PR review and serialized-codebase passes to catch logical errors and silent issues; one model reviewing another model's output to avoid self-review bias | Human review                                                               |


### AI Usage Policy

Describe any AI usage policies, guidelines, or constraints your team followed during development. This may also include course-specific requirements, or internal team agreements.

| Policy/Guideline        | Description                                                                                 | Application                                                                                                |
|-------------------------|---------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------|
| Privacy & Data Security | No proprietary data; do not feed personal bound information into models.                    | Prevented usage of live user data or sensitive material into prompts.                                      |
| Latest Tool Usage       | Guarantee use of the latest LLM Models (Gemini 3.1 Pro, GPT 5.5, Opus 4.7).                 | Ensured our outputs leveraged state-of-the-art capability for better logic results.                        |
| AI Generation Rule      | Do not code large parts yourself, optimize prompts to implement code in a shorter timespan. | Addressed the core course objective by evaluating LLMs' coding capabilities instead of manual keystroking. |


---

## Key Results

### What Worked Well

- The mode-aware engine configuration (`GameMode` / `GameConfig` / `Ruleset` / `BoardGeometry`) let Blokus Duo land as a small, additive change across all three layers instead of a rewrite — the headline result of the change request.
- A strict CI pipeline (`make check`: formatting, linting, strict type checking, tests, and a coverage gate) enforced from day one kept quality high even though most of the code was LLM-generated.
- The typed boundary — the engine's PyO3 `.pyi` stub plus the hand-written `PROTOCOL.md` — kept the Rust, Python, and Dart layers in sync and made grounded code generation reliable.
- Cross-instance LLM review, where one model critiqued another model's output, caught bugs that a single model missed.

### What Failed or Was Challenging

- Without repository-grounding context, LLMs invented project-specific APIs and type signatures that looked plausible but did not exist.
- Green tests and high coverage masked silent defects — a concurrency bug in the AI loop that blocked the event loop, and remediation loops that were tempted to weaken assertions instead of fixing the code.
- Agentic-coding usage limits (Codex, Claude Code) became restrictive over the project — effectively about one large task every few hours — which is hard to rely on for continuous work.
- Cross-platform frontend quirks: drag-and-drop worked on mobile but not on macOS, and the board rendered very small on phones.

### Lessons Learned

- Grounding context (type stubs, a minimal `AGENTS.md`) matters more than prompt cleverness for correctness.
- A strong automated validation pipeline is the precondition for agentic coding to be trustworthy — without it there is no reliable remediation signal.
- "Tests pass" certifies functional correctness, not non-functional properties such as concurrency and latency; those need their own checks.
- Define the needed abstraction deliberately and early — the LLMs did not anticipate the Classic-to-Duo generalization on their own, which was the deliberate point of the change request.

---

## Top 3 Counterexamples

Provide links to notable counterexamples where guidelines from other teams did not work as expected.

1. **Counterexample 1:** Hallucinated project APIs under Atomic Task Decomposition
   **Link:** [`Portfolio_Tobias_Friedrich.md`](../individual/Portfolio_Tobias_Friedrich.md) (Counterexample 1) and [`counterexample-atomic-decomposition-output.md`](../individual/evidence/counterexample-atomic-decomposition-output.md)
   **Guideline that Failed:** Coding G4 — Atomic Task Decomposition
   **What Happened:** Single-function prompts produced code that invented `Board` / `Piece` / `Position` types and ignored the crate's error conventions. Fixed by carrying repository context into each atomic prompt (Coding G1).

2. **Counterexample 2:** Passing tests certified a concurrency-broken AI loop
   **Link:** [`Portfolio_Aleksander_Kasak.md`](../individual/Portfolio_Aleksander_Kasak.md) (Counterexample 1), commit [`05aa9f9`](https://github.com/THF151/blocus-engine-cs-630/commit/05aa9f96922f22a4ffe8a2c0bb2cfa1ab110a263)
   **Guideline that Failed:** Coding G2: Interactive Test-Driven Validation (tests as source of truth)
   **What Happened:** A green, ~89%-covered suite passed an AI loop that ran up to 10,000 iterations without yielding and blocked the worker's event loop. Outcome assertions cannot express responsiveness, so "tests pass" certified the wrong thing.

3. **Counterexample 3:** Validate-and-repair masked a real defect
   **Link:** [`Portfolio_Tobias_Friedrich.md`](../individual/Portfolio_Tobias_Friedrich.md) (Counterexample 3) and [`Portfolio_Aleksander_Kasak.md`](../individual/Portfolio_Aleksander_Kasak.md) (Counterexample 3, commit [`0949b74`](https://github.com/THF151/blocus-engine-cs-630/commit/0949b74))
   **Guideline that Failed:** Testing validate-and-repair loop / Coding G3 — Iterative Remediation
   **What Happened:** When a generated test failed, the remediation loop "fixed" it by changing test data or weakening the assertion rather than fixing the system under test, which hid the underlying bug. The fix was to forbid the loop from editing assertions and require a human check of what the test still asserts.

> **Note:** The link to counterexample documentation can be any repository path or platform link (e.g., issue)

---

## Classic → Duo Change Request

### Impact on Design

How did the requirement to support Blokus Duo affect your design decisions?

For the engine core, the Duo change request showed that our first design was too focused on Classic Blokus. Several parts of the engine assumed four Classic colors, a 20×20 board, fixed corner starts, and Classic scoring. To support Duo cleanly, we changed the design so that the engine is configured by game mode instead of hard-coding one ruleset. When we discussed this before starting the implementation, we deliberately chose not to include the future duo mode into our requirements, as we wanted to find out if the LLMs would catch this level of abstraction by themselves (which they did not).

- **Initial Design Decisions:**
  The original engine was designed around Classic Blokus. It used the Classic colors blue, yellow, red, and green, a 20×20 board, fixed starting corners, and Classic turn order. This worked well for Classic two-player, three-player, and four-player modes, but it did not fully support game variants.
- **Changes Made for Duo Support:**
  We added a new `GameMode::Duo`, new player colors `Black` and `White`, a 14×14 Duo board, Duo starting points at `(4, 4)` and `(9, 9)`, Duo turn order, and advanced-only scoring. We also updated move generation, move validation, board validation, scoring, hashing, JSON serialization, Python bindings, examples, and tests to be aware of the selected game mode.
- **Challenges Encountered:**
  The biggest challenge was removing Classic-only assumptions. Color arrays had to support six stored color slots while each mode only uses its active colors. Board logic had to support both 20×20 Classic and 14×14 Duo. Opening move validation also had to change because Duo does not use the four Classic corners.
- **Solutions Implemented:**
  We introduced mode-aware configuration and rules. Each game mode now defines its active colors, board size, turn-order policy, opening policy, and scoring restrictions. Classic modes still use blue, yellow, red, and green on a 20×20 board. Duo uses black and white on a 14×14 board with two shared starting points. This allowed us to reuse the same core engine while supporting different rule variants.

### Configuration Approach

How did you implement configuration to support both Classic and Duo modes?

We implemented configuration through `GameMode`, `GameConfig`, `PlayerSlots`, `TurnOrder`, `Ruleset`, and `BoardGeometry`.

Each mode describes the rules it needs:

- Classic modes use the active colors `Blue`, `Yellow`, `Red`, and `Green`.
- Duo uses the active colors `Black` and `White`.
- Classic modes use a 20×20 board.
- Duo uses a 14×14 board.
- Classic opening moves must cover fixed corners.
- Duo opening moves must cover one of the two starting points, `(4, 4)` or `(9, 9)`.
- Duo only allows advanced scoring.

This approach kept the Rust core engine shared across all modes. Instead of creating a separate Duo engine, the same engine reads the mode configuration and applies the correct rules.
By keeping the API for the engine relatively stable, this only needed minimal changes for the python and flutter counterparts.
In a future iteration, the flutter frontend could call the rust engine directly for a singleplayer, therefore also directly adopting the new duo variant.

### Testing Strategy

How did you update your test suite to cover both modes?

We updated both the Rust and Python test suites to cover Classic and Duo behavior.

For Rust, we added and updated tests for:

- black and white player colors
- Duo turn order
- Duo board geometry
- Duo ruleset configuration
- Duo opening rules
- active-color validation
- mode-aware move generation
- scoring restrictions
- compact game state validation

For Python, we added tests for the public API:

- `GameMode.DUO`
- `PlayerColor.BLACK` and `PlayerColor.WHITE`
- `GameConfig.duo(...)`
- 14×14 board matrix output
- legal Duo opening moves
- rejection of invalid Duo openings
- rejection of moves outside the 14×14 board
- JSON round trips for Duo game states
- rejection of Basic scoring in Duo

This gave us regression coverage for the old Classic behavior and new coverage for Duo-specific behavior.

---

## Repository Links

- **Project Repository:** `https://github.com/THF151/blocus-engine-cs-630`
- **Issue Tracker:** `We did not use an issue tracker. Requirements were kept in markdown files.`
- **CI/CD Pipeline:** `https://github.com/THF151/blocus-engine-cs-630/blob/main/.github/workflows/ci.yml`

---

## Instructions for Use

1. **Replace all `[...]` placeholders** with your team's specific content
2. **Keep it concise** (1-2 pages max)
3. **Include one architecture diagram** (can be simple ASCII art or an image)
4. **Be honest about what worked and what didn't**
5. **Link to specific counterexamples** and documentation
6. **Reflect on how the change request affected your design**
7. **Submit as `Team_Summary.md`** in your project repository

---

*Template version: 1.0 | Last updated: 24 February 2026*
