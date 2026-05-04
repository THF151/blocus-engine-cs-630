_# OWNERSHIP.md

> **Team Project Ownership Table (who did what)**  
> *Document individual contributions and responsibilities for the Blokus Game Engine project.*

---

## Team Information

**Team Name:** Design team
**Project:** Blokus Game Engine (Classic + Duo)
**Date:** 3 May 2026
**Team Members:** Aleksander Kasak, Stephan Herbert, Tobias Friedrich

---

## Work Package Ownership

| Package Name          | Owner            | Responsibilities                                                                                                 | Acceptance Criteria                                          | Evidence Links               |
|-----------------------|------------------|------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------|------------------------------|
| Backend Service Layer | Aleksander Kasak | FastAPI service layer, exposing engine methods over HTTP/websockets, handling matchmaking and state persistence. | [See Backend AC](#backend-service-layer-acceptance-criteria) | [backend/](../../backend/)   |
| AI Players            | Aleksander Kasak | Simple simulation of artificial players.                                                                         | [See AI Players AC](#ai-players-acceptance-criteria)         | [backend/](../../backend/)   |
| Flutter Frontend      | Stephan Herbert  | UI design and implementation across platforms (ios/android/web/desktop), state rendering, player interaction     | [See Frontend AC](#flutter-frontend-acceptance-criteria)     | [frontend/](../../frontend/) |
| Rust Core Engine      | Tobias Friedrich | Core game logic, state management, move generation/validation, FFI bindings to Python                            | [See Core Engine AC](#rust-core-engine-acceptance-criteria)  | [engine/](../../engine/)     |

---

## Acceptance Criteria

### Backend Service Layer Acceptance Criteria
- Provides endpoints or FFI integrations allowing the frontend to act on the core engine securely.
- Connects the Rust core logic to network contexts efficiently.
- Integrates with an external state management system (such as Redis) to handle state persistence, ensuring cloud nativity and horizontal scalability.
- Ensures requests and states are handled smoothly without degrading performance.

### AI Players Acceptance Criteria
- Implements simple simulation of artificial players that can adhere to the game rules.
- Evaluates and selects legal moves automatically when it is the AI's turn.
- Integrates seamlessly with the backend game loop to replace missing human players.

### Flutter Frontend Acceptance Criteria
- Provides a welcoming initialization screen for game setup, player selection, and lobby creation.
- Supports online multiplayer, allowing users to play over the network with up to 4 other participants.
- Visually represents the Blokus (Classic/Duo) board sizes respectively.
- Exposes drag/drop or click-based interfaces to allow users to interact with the current piece and target cells.
- Syncs remaining times, scores, and piece lists from the engine data correctly.
- Transitions effectively to the score/finish screen after the engine signals a blocked/finished game state.

### Rust Core Engine Acceptance Criteria
- **Game Setup:** Properly initializes the board dimensions (14x14 for Duo, 20x20 for Classic) and seeds players with the standard set of 21 unique shapes. Supports appropriate modes for two, three, and four players along with basic and advanced scoring paradigms.
- **Core Placement Rules:** Enforces the fundamental Blokus placement rules: pieces of the same color must touch at the corners but cannot share flat edges. Ensures that initial plays begin on the correct designated starting squares.
- **Move Validation:** Evaluates requested placements accurately, accounting for piece rotation and reflection, and rejects any invalid moves gracefully without crashing the game state.
- **Turn Tracking & Passing:** Tracks player turns and correctly manages passing for players who are blocked or out of pieces. Detects when all players are completely blocked to correctly trigger the end of the game.
- **Rules Scoring Calculation:** Correctly computes scores based on the unplayed pieces (-1 point per square). Accurately applies bonuses for completing the board (+15 points) and finishing with the 1-square piece (+5 extra points) for advanced scoring.
- **Rule Adherence** In general adheres to the official classic and duo rules.
