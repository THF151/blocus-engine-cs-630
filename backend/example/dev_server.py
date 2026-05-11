"""Dev launcher for the FastAPI backend with in-memory repo + event bus.

The default ``make dev`` target wires Redis. For local CLI experiments
(e.g. ``visual_simulation.py``) where Redis isn't running, this script
spins up the same app on http://127.0.0.1:8000 with all state held in
process. CAS, seat binding, pubsub fan-out — everything still works,
just bounded to a single process.

Usage:
    uv run python backend/example/dev_server.py
    uv run python backend/example/dev_server.py --port 9000

Then point ``visual_simulation.py`` (or any client) at
``ws://localhost:8000/ws``.
"""

from __future__ import annotations

import argparse

import uvicorn

from blocus_backend.app import create_app
from blocus_backend.event_bus import InMemoryGameEventBus
from blocus_backend.repository import InMemoryGameRepository


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--log-level", default="info")
    args = parser.parse_args()

    app = create_app(
        repository=InMemoryGameRepository(),
        event_bus=InMemoryGameEventBus(),
    )
    uvicorn.run(app, host=args.host, port=args.port, log_level=args.log_level)


if __name__ == "__main__":
    main()
