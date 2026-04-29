import blocus_engine
from fastapi import FastAPI

app = FastAPI(title="Blocus Backend")


@app.get("/health")
def health() -> dict[str, bool | str]:
    return {
        "status": "ok",
        "engine": blocus_engine.engine_health(),
    }
