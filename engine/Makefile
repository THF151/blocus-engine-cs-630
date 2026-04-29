.PHONY: help install fmt fmt-check lint test test-rust test-python coverage coverage-html check clean

help:
	@echo "Available commands:"
	@echo ""
	@echo "  make install       Install Rust formatting/linting/coverage tools"
	@echo "  make fmt           Format Rust code"
	@echo "  make fmt-check     Check Rust formatting"
	@echo "  make lint          Run Clippy linting"
	@echo "  make test          Run Rust core and Python binding tests"
	@echo "  make test-rust     Run Rust core tests"
	@echo "  make test-python   Build Python extension and run Python binding tests"
	@echo "  make coverage      Run Rust core coverage"
	@echo "  make coverage-html Run Rust core coverage and open HTML report"
	@echo "  make check         Run fmt-check, lint, coverage, and Python binding tests"
	@echo "  make clean         Remove Rust build artifacts"

install:
	rustup component add rustfmt clippy llvm-tools-preview
	cargo install cargo-llvm-cov

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy -p blocus-core --all-targets --all-features -- -D warnings
	cargo clippy -p blocus-python --lib --all-features -- -D warnings

test-rust:
	cargo test -p blocus-core

test-python:
	cd ../backend && uv run maturin develop --manifest-path ../engine/crates/blocus-python/Cargo.toml
	cd ../backend && uv run pytest ../engine/crates/blocus-python/python_tests

test: test-rust test-python

coverage:
	cargo llvm-cov -p blocus-core --fail-under-lines 85

coverage-html:
	cargo llvm-cov -p blocus-core --html --open

check: fmt-check lint coverage test-python

clean:
	cargo clean