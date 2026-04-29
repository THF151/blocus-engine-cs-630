.PHONY: help install format format-check analyze test coverage check clean

help:
	@echo "Available commands:"
	@echo ""
	@echo "  make install       Install Flutter dependencies"
	@echo "  make format        Format Dart code"
	@echo "  make format-check  Check Dart formatting"
	@echo "  make analyze       Run Flutter static analysis"
	@echo "  make test          Run Flutter tests"
	@echo "  make coverage      Run Flutter tests with coverage"
	@echo "  make check         Run format-check, analyze, and tests"
	@echo "  make clean         Clean Flutter build artifacts"

install:
	flutter pub get

format:
	dart format .

format-check:
	dart format --set-exit-if-changed .

analyze:
	flutter analyze

test:
	flutter test

coverage:
	flutter test --coverage

check: format-check analyze test

clean:
	flutter clean
