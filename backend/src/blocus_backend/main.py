from fastapi import FastAPI

app = FastAPI(title="Blocus Backend")


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}
