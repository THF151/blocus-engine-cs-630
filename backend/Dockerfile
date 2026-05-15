# syntax=docker/dockerfile:1.7

FROM python:3.14-slim AS builder

ARG RUST_VERSION=1.95.0

ENV PATH="/root/.cargo/bin:/root/.local/bin:/opt/venv/bin:${PATH}" \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    UV_COMPILE_BYTECODE=1 \
    UV_LINK_MODE=copy \
    UV_PROJECT_ENVIRONMENT=/opt/venv

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN curl -LsSf https://astral.sh/uv/install.sh | sh

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y \
        --default-toolchain "${RUST_VERSION}" \
        --profile minimal \
    && rustup default "${RUST_VERSION}"

COPY pyproject.toml uv.lock README.md ./
COPY src ./src

RUN uv sync --frozen --no-dev --no-editable \
    && uv pip install --python /opt/venv/bin/python gunicorn

RUN uv pip install --system "maturin>=1.13.1"

COPY --from=engine-context Cargo.toml Cargo.lock /engine/
COPY --from=engine-context crates/blocus-core/Cargo.toml /engine/crates/blocus-core/Cargo.toml
COPY --from=engine-context crates/blocus-python/Cargo.toml /engine/crates/blocus-python/Cargo.toml
COPY --from=engine-context crates/blocus-core/src /engine/crates/blocus-core/src
COPY --from=engine-context crates/blocus-python/src /engine/crates/blocus-python/src

RUN maturin build --release \
        --manifest-path /engine/crates/blocus-python/Cargo.toml \
        --out /tmp/wheels \
    && uv pip install --python /opt/venv/bin/python /tmp/wheels/*.whl

FROM python:3.14-slim AS runtime

ENV PATH="/opt/venv/bin:${PATH}" \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    BLOCUS_REDIS_URL=redis://redis:6379/0

WORKDIR /app

COPY --from=builder /opt/venv /opt/venv

RUN useradd --create-home --shell /usr/sbin/nologin appuser

USER appuser

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8000/health', timeout=3).read()"

CMD ["gunicorn", "blocus_backend.main:app", "--worker-class", "uvicorn.workers.UvicornWorker", "--workers", "4", "--bind", "0.0.0.0:8000"]
