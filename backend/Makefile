.PHONY: help install dev run run-uvicorn test coverage lint format typecheck check clean

help:

	@echo "Available commands:"
	@echo ""
	@echo "  make install       Install/sync dependencies with uv"
	@echo "  make dev           Start FastAPI dev server with auto-reload"
	@echo "  make run           Start production server with fastapi run"
	@echo "  make run-uvicorn   Start server directly with uvicorn"
	@echo "  make test          Run tests"
	@echo "  make coverage      Run tests with coverage report"
	@echo "  make lint          Run Ruff linting"
	@echo "  make format        Format code"
	@echo "  make typecheck     Run mypy"
	@echo "  make check         Run lint, typecheck, and coverage"
	@echo "  make clean         Remove caches and coverage artifacts"

install:
	uv sync

dev:
	uv run fastapi dev src/blocus_backend/main.py

run:
	uv run fastapi run src/blocus_backend/main.py

run-uvicorn:
	uv run uvicorn blocus_backend.main:app --host 0.0.0.0 --port 8000

test:
	uv run pytest

coverage:
	uv run pytest --cov=blocus_backend --cov-report=term-missing --cov-report=html

lint:
	uv run ruff check .

format:
	uv run ruff format .

typecheck:
	uv run mypy src

check: lint typecheck coverage

clean:
	rm -rf .pytest_cache .ruff_cache .mypy_cache htmlcov .coverage coverage.xml
	find . -type d -name "__pycache__" -exec rm -rf {} +