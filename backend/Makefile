.PHONY: help install build-engine dev run run-uvicorn test coverage lint format typecheck check clean

help:
	@echo "Available commands:"
	@echo ""
	@echo "  make install       Install/sync dependencies with uv"
	@echo "  make build-engine  Build/install Rust Python engine binding"
	@echo "  make dev           Build engine and start FastAPI dev server"
	@echo "  make run           Build engine and start production server with fastapi run"
	@echo "  make run-uvicorn   Build engine and start server directly with uvicorn"
	@echo "  make test          Build engine and run tests"
	@echo "  make coverage      Build engine and run tests with coverage report"
	@echo "  make lint          Run Ruff linting"
	@echo "  make format        Format code"
	@echo "  make typecheck     Run mypy"
	@echo "  make check         Build engine, then run lint, typecheck, and coverage"
	@echo "  make clean         Remove caches and coverage artifacts"

install:
	uv sync

build-engine:
	uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml

dev: build-engine
	uv run fastapi dev src/blocus_backend/main.py

run: build-engine
	uv run fastapi run src/blocus_backend/main.py

run-uvicorn: build-engine
	uv run uvicorn blocus_backend.main:app --host 0.0.0.0 --port 8000

test: build-engine
	uv run pytest

coverage: build-engine
	uv run pytest --cov=blocus_backend --cov-report=term-missing --cov-report=html

lint:
	uv run ruff check .

format:
	uv run ruff format .

typecheck:
	uv run mypy src

check: build-engine lint typecheck coverage

clean:
	rm -rf .pytest_cache .ruff_cache .mypy_cache htmlcov .coverage coverage.xml
	find . -type d -name "__pycache__" -exec rm -rf {} +
