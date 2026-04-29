from fastapi import FastAPI

import blocus_engine

app = FastAPI(title="Blocus Backend")


@app.get("/health")
def health() -> dict[str, bool | str]:
    return {
        "status": "ok",
        "engine": blocus_engine.engine_health(),
    }
