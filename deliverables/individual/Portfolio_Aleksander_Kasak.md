# Portfolio_Aleksander_Kasak.md

> **Individual Student Portfolio**  
> *~4-5 page report documenting your contributions, guideline applications, and counterexamples.*

---

## Student Information

**Student Name:** `Aleksander Kasak`  
**Team Name:** `Design`  
**Project:** Blokus Game Engine (Classic + Duo)  

---

## 1. Owned Package Contributions

### Package Name: `blocus-backend`

**Description:**
I owned the FastAPI backend in `backend/`. 
It:
- exposes the Rust engine over a single WebSocket endpoint
- persists game state in Redis (with an in-memory fallback for `make dev`), and fans 
- broadcasts out to every subscribed client
The the rules between Rust engine and the Flutter frontend them are in `PROTOCOL.md` 

**Responsibilities:**
- expose the engine over `/ws`
- persist game state in Redis 
- broadcast state changes via Pub/Sub
- map the four game modes onto engine Interface

**Evidence Links:**
- **Package:** [`backend/src/blocus_backend/`](../../backend/src/blocus_backend/)
- **Wire protocol:** [`backend/PROTOCOL.md`](../../backend/PROTOCOL.md)
- **Tests:** [`backend/tests/`](../../backend/tests/)


**Key Contributions:**
- Implemented the WebSocket dispatcher and `ConnectionManager` (seat binding, takeover on reconnect, broadcast that handles a dropped subscriber)
- Pydantic schemas at the wire boundary so validation errors return typed codes
- Added optimistic concurrency via a Redis Lua CAS script on `state.version`.
- Pub/Sub event bus with backoff and stream replay
- Wrote the engine adapter for all four modes, including the Duo extension
- Added the `player_joined` broadcast for the lobby
- Wrote `PROTOCOL.md` and the Docker Compose stack

---

### Package Name: AI Players

**Description:**
The AI players are included directly into the backend service layer because the engine already exposes a cheap legal-move iterator. The backend tracks which `(game_id, color)` pairs are AI-controlled, and after every applied move checks whether the next color is AI. If yes, it asks the engine for the first legal move, applies it and broadcasts a result. The loop yields between turns (`asyncio.sleep(0)`)

**Responsibilities:**
- Provide an `attach_ai` action.
- AI loop after every move until the current color is human-controlled again.
- Pick moves through the engine
- seat-binding so a human can always reclaim an AI spot


**Evidence Links:**
- **Commits:** [`6a5945b`](https://github.com/THF151/blocus-engine-cs-630/commit/6a5945b2df5011b684b7dae78e06ebcff441a87d) (`attach_ai` / `advance_ai_turns`), [`05aa9f9`](https://github.com/THF151/blocus-engine-cs-630/commit/05aa9f96922f22a4ffe8a2c0bb2cfa1ab110a263)
- **Tests:** [`backend/tests/test_websocket_classic.py`](../../backend/tests/test_websocket_classic.py) (`test_attach_ai_plays_first_legal_move_for_current_turn`, `test_ai_turns_continue_until_next_non_ai_seat`)
- **Documentation:** `attach_ai` action in [`backend/PROTOCOL.md`](../../backend/PROTOCOL.md)

**Key Contributions:**
- Added the `attach_ai` wire action with a `game_joined` broadcast 
- Implemented AI loop that plays multi-AI games to completion
- seat-binding: a human seat always beats an AI attachment, so a player can reclaim mid-game by re-subscribing

---

## 2. Guideline Applications

> **Note:** Document at least 3 applications of guidelines from other teams' guideline packages. For each, describe the guideline, how you applied it, and the outcome.

### Application 1: `Iterative Remediation and Self-Correction Loops (G3)` from `Coding` Team

**Guideline Description:**
Coding G3 says to treat the first generated output as a draft, not a finished result. Failed test output, tracebacks and lint errors should be fed back into the model so it can remediate, and a secondary "reviewer" turn should critique the code for silent hallucinations — bugs, security holes, or incomplete behavior that pass syntax and tests. The premise is that models are better at finding errors in existing text than at avoiding them during generation.

**Context:**
I applied this throughout the backend build. Almost every module (`service.py`, `websocket.py`, `repository.py`, `event_bus.py`) started as an LLM draft from Codex or Claude Code, then went through `make check` (ruff + mypy + pytest + coverage) as the remediation signal. The pipeline made the loop concrete: a failure is not an opinion, it is a traceback I can paste back.

**Application Process:**
1. Treated each generated patch as a draft and ran `make check` immediately. Failures (assertion, type, lint) went straight back to the model with the traceback.
2. Capped remediation at about two attempts. If the second pass still failed, I fixed it by hand rather than looping further — past two rounds the advice gets repetitive.
3. For the reviewer turn I sometimes used a *second instance*: Codex critiquing Claude Code's output or the other way around, to get an independent read and avoid a model rubber-stamping its own code.
4. Concrete loop in action: adding the `player_joined` broadcast (commit `0949b74`) broke several existing tests that assumed `state_snapshot` arrives before the broadcast. The loop surfaced the failures, the diagnosis was that the broadcast fires before the unicast snapshot, and the fix was to consume `player_joined` first in those tests.

**Outcome:**
- **What worked:** the loop caught the obvious failures fast, and the cross-instance reviewer caught things the generating model was blind to — a different model reviewing the code is meaningfully better than asking the author model "are you sure?"
- **What didn't work:** the reviewer turn was not systematic. Most code went loop-to-green only, and iterate-to-green optimizes for "tests pass," which can mask silent issues. The sharpest version is a remediation that weakens a failing assertion instead of fixing the code: green pipeline, worse test
- **Evidence:** the `player_joined` remediation in commit [`0949b74`](https://github.com/THF151/blocus-engine-cs-630/commit/0949b74) and the test edits it forced; the `make check` gate in [`backend/Makefile`](../../backend/Makefile) and [`backend/pyproject.toml`](../../backend/pyproject.toml)

**Reflection:**
The loop is cheap and works as long as the failure signal is honest. The part I underused was the reviewer turn, and the refinement worth keeping is that it should be a *different* model or instance — a model reviewing its own output is the weak case the guideline is really warning about. Next time I would make the cross-instance review a fixed step before merge, not an occasional one.

---

### Application 2: `Proactively Detect and Resolve Ambiguity Through Clarification (G3)` from `Requirements` Team

**Guideline Description:**
Requirements G3 says that before generating an artifact, the LLM should flag the statements it finds ambiguous, ask short clarification questions, and then embed the resolved answers back into the output rather than quietly guessing. The guideline frames this for requirements elicitation, but the shape fits any spec artifact whose inputs are partly informal: the point is to surface hidden decisions instead of letting the model smooth them over.

**Context:**
I applied this when drafting `PROTOCOL.md`, the wire-protocol document Stephan uses as the typed contract for the Flutter frontend. The inputs were the working FastAPI service layer and the Pydantic schemas (which already fix payload structure), plus a set of design decisions that were *not* in the code: `command_id` idempotency, whether the server retries CAS conflicts for the client, the trust model, seat-takeover semantics, and the versioning policy. These are exactly the points a code-reader cannot infer and where a one-shot draft would have invented an answer.

**Application Process:**
1. Gave the model the service code and the Pydantic schemas, and instructed it: "before writing the document, list every question whose answer is not derivable from the code. Do not draft yet."
2. Got back ~15 questions. Roughly a third were genuine open decisions, a third were things I had decided but never written down, and a third were noise already answered by the code.
3. Answered the real ones in a flat decisions block and fed them back as the grounding for the draft.
4. Reviewed the draft against the code and split one overlapping error code (`internal_error` vs `engine_unavailable`).

**Outcome:**
- **What worked:** the clarification round forced out decisions that then became concrete, named sections instead of vague prose. The proof is that each surfaced ambiguity has a discrete documented home in `PROTOCOL.md`:
  - `command_id` as the engine idempotency key — `PROTOCOL.md:199`
  - CAS contract (clients never send `expected_version`; server retries for AI, surfaces `conflict` for humans) — `PROTOCOL.md:399–405`
  - Trust model (no auth; per-connection seat binding is the only identity guarantee; MITM forgery explicitly out of scope) — `PROTOCOL.md:413–430`
  - Seat-takeover semantics (`kicked` / `seat_taken_by_reconnect`) — `PROTOCOL.md:54–58`
  - Versioning policy (`/ws` is v1; breaking changes go to `/ws/v2`) — `PROTOCOL.md:434–442`
- **What didn't work:** about a third of the generated questions were already answered by the code, so I filtered them by hand. The model asks broadly rather than reading the obvious answers out of the source first
- **Evidence:** the document itself, [`backend/PROTOCOL.md`](../../backend/PROTOCOL.md), with the section-by-section mapping above. Each cited line is a decision that exists *only* because the clarification step pulled it out before drafting.

**Reflection:**
The clarification step is cheap and the payoff is high for a contract that another person builds against. Writing 15 short answers cost a few minutes; a protocol doc that silently disagreed with the running code would have cost Stephan a debugging afternoon. Next time I would pre-feed the serialized codebase so the model can answer the derivable questions itself and only ask about the genuine gaps, which would cut the third of the list that was noise.

---

### Application 3: `Context-Aware Grounding via Minimal Manual Documentation (G1)` from `Coding` Team

**Guideline Description:**
Coding G1 says generation should be grounded in project-specific context — a minimal manual context file (`AGENTS.md`), the public APIs of the surrounding packages, and the type stubs the new code has to call into — rather than letting the model work from the task description alone. The reasoning is that most real functions depend on project entities the model will otherwise hallucinate.

**Context:**
I applied this when generating the first end-to-end version of the backend service layer. The job was to expose Tobias's engine over WebSockets, and the engine already existed as a PyO3 binding with a type stub (`backend/src/blocus_engine.pyi`). The FastAPI side was an near-empty scaffold, so the model had to write `service.py`, `engine_adapter.py` and `websocket.py` against an API it could easily get wrong.

**Application Process:**
1. Bundled the engine type stub (`blocus_engine.pyi`) into the prompt context. The stub gives the model the real surface — `BlocusEngine.apply`, `get_valid_moves`, `GameConfig.two_player` — instead of letting it guess from the Rust source.
2. Added a short context note for the project conventions (the Repository / EventBus interfaces, the `ProtocolError` to typed-wire-error mapping).
3. Asked the model to plan first, then generate file by file, running `make check` between steps.
4. Reviewed each file against the stub before moving on.

**Outcome:**
- **What worked:** with the `.pyi` in context the model called the real engine methods with the right signatures on the first try. `engine_adapter.py` and `service.py` came out close to final.
- **What didn't work:** an earlier attempt without the stub invented an `Engine.create_game(mode, players)` and a `state.serialize()`, neither of which exist. They look plausible and would have cost a review cycle. The fix was just feeding the stub in — same prompt, real APIs
- **Evidence:** the engine stub used as context, [`backend/src/blocus_engine.pyi`](../../backend/src/blocus_engine.pyi), and the generated package [`backend/src/blocus_backend/`](../../backend/src/blocus_backend/)

**Reflection:**
For code that calls into a typed boundary — a PyO3 stub, a `.proto`, an OpenAPI client — a single interface file is worth more than a page of prose instructions. The grounding context decides correctness; the prose decides style. I would put the type surface first in the prompt every time, because the model anchors hardest on whatever it sees first.

---

## 3. Counterexamples

> **Note:** Document at least 3 reproducible counterexamples where guidelines failed or produced suboptimal results. For each, include the failure, diagnosis, and refinement.

### Counterexample 1: Passing tests certified a concurrency-broken AI loop (`Coding G2`)

**Failure Description:**
Coding G2 (Interactive Test-Driven Validation) treats human-verified tests as the source of truth: code that passes them is pruned-in as correct. I generated `advance_ai_turns` — the loop that plays attached AI seats after a move — and it passed the whole suite. The suite was green at ~89% coverage, so by G2's premise the code was correct. It was not. The loop ran up to `MAX_AI_TURNS = 10_000` iterations with no explicit yield point, so a long all-AI chain monopolised the worker's event loop and starved the very WebSocket connection that triggered it. Every assertion in the suite is about *game outcomes* (whose turn, which move, final score), and all of those were correct — the defect was that the worker stopped answering anyone else while the chain ran. No test failed, because no test could express "the event loop stays responsive."

**Diagnosis:**
- **Root Cause:** the loop only yielded at real I/O awaits; under the in-memory repository (and fast Redis) those complete synchronously, so 10,000 iterations could run without ever returning control to the event loop.
- **Why the Guideline Failed:** G2's premise is that passing tests prune incorrect code. That holds for functional correctness but not for non-functional properties (concurrency, latency, responsiveness), which assertion-based unit tests do not capture. The tests were the wrong instrument, so "passes the tests" certified the wrong thing.
- **Boundary Condition:** reproducible whenever the property under test is temporal/concurrency-related rather than input→output. Tests-as-truth is safe for pure logic, unsafe for event-loop and scheduling behaviour.

**Refinement:**
- **Updated Guideline:** keep G2 for functional correctness, but for any async/concurrent code add an explicit responsiveness assertion to the "source of truth" — e.g. a test that the loop yields each iteration — rather than trusting outcome assertions to cover it.
- **How It Was Tested (evaluated):** I added `test_advance_ai_turns_yields_to_event_loop`, which monkeypatches `asyncio.sleep` and asserts a zero-delay sleep is awaited per iteration, then added `await asyncio.sleep(0)` to the loop. The new test fails against the old code and passes against the fixed code.
- **Evidence:** commit [`05aa9f9`](https://github.com/THF151/blocus-engine-cs-630/commit/05aa9f96922f22a4ffe8a2c0bb2cfa1ab110a263) (fix + the yield test).

**Prompt/Context Used:**
```
Implement advance_ai_turns(game_id): after a move is applied, if the next
color is an attached AI seat, play its first legal move and broadcast the
result, repeating until a human-controlled color or the game ends. Cap the
loop at MAX_AI_TURNS. Add tests.
```

**AI Output:**
```python
async def advance_ai_turns(self, game_id: str) -> list[dict[str, Any]]:
    events: list[dict[str, Any]] = []
    for _ in range(MAX_AI_TURNS):          # up to 10_000 iterations
        record = await self._record(game_id)   # only yields at real I/O
        ...
        result = self._engine.place_move(state, command_payload)
        event = await self._persist_apply_result(game_id, record, result)
        events.append(event)
    return events
```
*Real pre-fix code, from the parent of commit `05aa9f9`. No `await asyncio.sleep(0)`; the suite passed against it.*

---

### Counterexample 2: Clarification step asked questions the code already answered (`Requirements G3`)

**Failure Description:**
Requirements G3 says to have the model flag ambiguities and ask clarification questions *before* drafting. I applied it to `PROTOCOL.md` (see Application 2) and it did surface real gaps — but roughly a third of the ~15 questions it asked were already answered by the code and the Pydantic schemas it had been given. It asked which fields `place_move` requires (fixed in `PlaceMoveRequest`), what the create-game modes are (the schema's discriminator), and what shape `legal_moves` returns. These are not ambiguities; they are facts the model could have read out of the inputs. The cost was small per question, but the noise diluted the genuinely open decisions (trust model, CAS visibility) and twice it re-opened a decision I had already made, which I then had to re-defend rather than just confirm.

**Diagnosis:**
- **Root Cause:** the model asks broadly to look thorough rather than first reading the answers out of the context it already has.
- **Why the Guideline Failed:** G3 assumes the inputs are informal and under-specified, so "flag everything unclear" is net-positive. When a large part of the contract is already pinned by code and typed schemas, the same instruction generates questions whose answers are in front of it. The guideline does not tell the model to *exhaust the provided source* before asking.
- **Boundary Condition:** reproducible whenever the artifact's inputs already encode most of the answer (code-grounded specs, typed APIs). G3 stays valuable for the genuinely open subset, but the signal-to-noise drops as the inputs get more formal.

**Refinement:**
- **Updated Guideline:** add a pre-pass to G3 — instruct the model to first answer every question it can from the provided code/schemas and only surface the ones that remain. "Ask what you cannot derive," not "ask what is unclear."
- **How It Was Tested (evaluated):** on a later protocol edit I prepended "answer from the code first; only list what is genuinely undecided." The question list shrank to the real decisions, and no already-settled item came back.
- **Evidence:** [`backend/PROTOCOL.md`](../../backend/PROTOCOL.md) and the schemas it is grounded in, [`backend/src/blocus_backend/schemas.py`](../../backend/src/blocus_backend/schemas.py).

**Prompt/Context Used:**
```
[context: service.py + schemas.py]
Before writing PROTOCOL.md, list every question whose answer is not
derivable from the code. Do not draft yet.
```

**AI Output:**
```
1. What is the trust model — is player_id authenticated?        [real gap]
2. Does the client send expected_version, or does the server     [real gap]
   track it?
3. What fields does place_move require?                          [in PlaceMoveRequest]
4. Which game modes can create_game take?                        [in the schema discriminator]
5. What does request_legal_moves return?                         [derivable from legal_moves]
... (~15 total; roughly a third were already answered by the inputs)
```

---

### Counterexample 3: Remediation loop "fixed" a test by muting its assertion (`Coding G3`)

**Failure Description:**
Coding G3's remediation loop feeds a failing test back to the model and asks it to make the suite green. The danger is that "make it pass" has two solutions: fix the code, or weaken the test. When I added the `player_joined` broadcast (commit `0949b74`), several existing tests broke because they asserted `state_snapshot` is the first message a subscriber receives — but the new broadcast fires *before* the unicast snapshot, so the real first message is now `player_joined`. The correct fix is to consume `player_joined` first and keep asserting the snapshot arrives after it. A naive remediation loop, told only "the suite is red, fix it," takes the cheaper path: it deletes or relaxes the offending assertion until the test goes green, which silently drops the ordering guarantee the test existed to protect. In this case I caught it and fixed the tests properly; the failure mode is what the loop *would* have done unsupervised.

**Diagnosis:**
- **Root Cause:** "make the suite pass" is underspecified — relaxing the assertion satisfies it as well as fixing the code, and the relaxation is the lower-effort path the model gravitates to.
- **Why the Guideline Failed:** G3 uses "tests pass" as the loop's termination signal without distinguishing a test that passes because the behaviour is right from one that passes because the check was removed. The loop optimises the signal, not the intent behind it.
- **Boundary Condition:** reproducible whenever the failing test encodes an invariant the change genuinely altered (here, message ordering). For a test failing on a trivial mismatch the shortcut is harmless; for an invariant-bearing test it erases coverage.

**Refinement:**
- **Updated Guideline:** forbid the remediation loop from editing assertions. Constrain it to either change non-test code or explain why the test itself is wrong — never silently weaken an assertion to reach green. Pair every red-to-green remediation with a human check of *what the test still asserts*.
- **How It Was Tested (evaluated):** I fixed the broken tests by hand to consume `player_joined` before the snapshot, preserving the ordering assertion, and confirmed the suite (70 tests) passes for the right reason.
- **Evidence:** commit [`0949b74`](https://github.com/THF151/blocus-engine-cs-630/commit/0949b74) — the tests that now read `player_joined` then `state_snapshot` instead of dropping the check.

**Prompt/Context Used:**
```
These tests now fail after I added the player_joined broadcast:
  assert owner.receive_json()["type"] == "state_snapshot"   # AssertionError: got "player_joined"
Make the test suite pass.
```

**AI Output:**
```python
# the cheap "fix": assertion relaxed instead of ordering understood
event = owner.receive_json()
assert event["type"] in {"player_joined", "state_snapshot"}   
```
*Reproduction of the failure mode. The committed fix (`0949b74`) instead consumes `player_joined` first and keeps the strict `state_snapshot` assertion.*

---

## 4. AI Usage Disclosure

### Tools and Models Used

| Tool/Model | Usage | Validation Method |
|------------|-------|-------------------|
| Claude Code (Opus 4.7) | Backend code + test generation; `PROTOCOL.md` clarify-first drafting; cross-instance review of Codex output | `make check` (ruff + mypy + pytest + coverage ≥85%), human review |
| Codex (GPT-5.x) | Backend code + test generation; cross-instance review of Claude Code output | `make check`, human review |
| GPT 5.5 Chat | Planning and design discussion; reasoning through the protocol decisions | Manual verification |
| GitHub Copilot | Occasional inline completion (one or two minor changes) | `make check`, human review |

### Evaluation Methods

1. **Correctness Testing:** every change ran through `make check` — ruff, mypy (strict), pytest, and a coverage gate of ≥85%. The same pipeline runs in CI on pushes and PRs.
2. **Code Review:** I reviewed every generated patch by hand, and for harder changes ran a cross-instance pass (Codex critiquing Claude Code's output or the reverse) so the reviewing model was not the author model.
3. **Unit Tests:** the WebSocket, service, repository and event-bus suites (70 tests total). Generated tests were reviewed for what they actually assert, not just whether they pass.
4. **Integration Tests:** multi-connection `TestClient` scenarios (broadcast, seat takeover, AI chains) and cross-`ConnectionManager` tests that share one repository and event bus to exercise the multi-worker path.
5. **Performance Testing:** light — a targeted test that the AI loop yields to the event loop each iteration (`test_advance_ai_turns_yields_to_event_loop`). I did not set up load or latency benchmarking, so this is the one area I would not overstate.

### Time Investment

Approximately how much time I spent on:
- AI prompting and refinement: `13 hours`
- Reviewing AI outputs: `8 hours`
- Testing and validation: `5 hours`
- Documentation (`PROTOCOL.md` + this portfolio): `10 hours`

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
