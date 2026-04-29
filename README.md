# Blocus Engine — CS-630

Group repository of master’s students for the **CS-630 course at the University of Mannheim**.

The project will become a Blocus engine with a Flutter frontend, FastAPI backend, and Rust core engine.

## Structure

```text
.
├── frontend/   # Flutter app
├── backend/    # FastAPI service
├── engine/     # Rust core + Python bindings
└── README.md
```

## Setup

### Frontend

```bash
cd frontend
flutter pub get
make check
```

### Backend
The backend depends on the Rust Python binding. The backend Makefile builds it automatically.

```bash
cd backend
uv sync
make check
```

### Engine

```bash
cd engine
make install
make check
```

## All Checks

From the repository root:

```bash
make check
```