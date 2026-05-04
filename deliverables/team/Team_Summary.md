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

The project implements a full-stack Blokus Game Engine encompassing both Classic and Duo variants. The tech stack focuses on a **Rust** core engine for game logic and move validation, integrated via FFI into a **Python (FastAPI)** backend service. The backend architecture is cloud-native, orchestrating multiplayer connections via HTTP/WebSockets and utilizing **Redis** for robust state management. Players interact through a **Flutter** frontend that supports seamless multi-platform gameplay (iOS/Android/Web/Desktop).

### Architecture Diagram

```mermaid
graph TD
    F[Flutter Frontend] <-->|HTTP / WebSockets| B[FastAPI Backend Service]
    B -->|FFI| R[Rust Core Engine]
    B <-->|State Persistence| Redis[(Redis)]
    A[AI Players] -->|Simulated Input| B
    F -.->|FFI - Future Singleplayer| R
```

### Key Components

1. **Rust Core Engine:** Handles core rules, board dimensions, legal placement evaluations, turn tracking, and scoring calculations.
2. **Backend Service Layer:** Manages request endpoints, WebSocket connections, concurrency, and interfaces directly with the Rust engine.
3. **State Management:** Redis instance used specifically for horizontal scalability and managing active lobby/game states efficiently in the cloud.
4. **AI Players:** Embedded simulation logics allowing automated agents to participate inside the backend loops.
5. **Flutter Frontend:** Provides the interactive user interface, rendering lobbies, and facilitating real-time drag-and-drop gameplay syncing with the backend.

---

## AI Tools Used

### High-Level Overview

Describe which AI tools you used and where. Be specific about the tools/models and how they were integrated into your workflow.

| Phase                          | AI Tool/Model                                    | Usage                                                   | Validation Method                                   |
|--------------------------------|--------------------------------------------------|---------------------------------------------------------|-----------------------------------------------------|
| Requirements                   | GPT 5.5, GPT 5.4, Gemini 3.1 Pro                 | Requirements extraction and specification building      | Manual verification, LLM as a judge                 |
| Design                         | Gemini 3.1 Pro, GPT 5.5                          | LLM ADR generation                                      | Manual review                                       |
| Code Gen & Testing & Debugging | Codex, Claude Code, GitHub Copilot, GPT 5.5 Chat | FFI bindings, core logic, Flutter UI, and test suites   | Manual code execution, test suites, LLM code review |
| Code Review                    | GPT 5.5                                          | Automated pull request PR feedback and structure checks | Manual verification                                 |


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

- `[Result 1]`
- `[Result 2]`
- `[Result 3]`

### What Failed or Was Challenging

- `[Challenge 1]`
- `[Challenge 2]`
- `[Challenge 3]`

### Lessons Learned

- `[Lesson 1]`
- `[Lesson 2]`
- `[Lesson 3]`

---

## Top 3 Counterexamples

Provide links to notable counterexamples where guidelines from other teams did not work as expected.

1. **Counterexample 1:** `[Title]`  
   **Link:** `[Link to counterexample documentation]`  
   **Guideline that Failed:** `[Name of guideline]`  
   **What Happened:** `[Brief description]`

2. **Counterexample 2:** `[Title]`  
   **Link:** `[Link to counterexample documentation]`  
   **Guideline that Failed:** `[Name of guideline]`  
   **What Happened:** `[Brief description]`

3. **Counterexample 3:** `[Title]`  
   **Link:** `[Link to counterexample documentation]`  
   **Guideline that Failed:** `[Name of guideline]`  
   **What Happened:** `[Brief description]`

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
