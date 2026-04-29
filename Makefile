.PHONY: help backend engine check clean

help:
	@echo "Available commands:"
	@echo ""
	@echo "  make backend   Run backend checks"
	@echo "  make engine    Run engine checks"
	@echo "  make check     Run all checks"
	@echo "  make clean     Clean backend and engine artifacts"

backend:
	$(MAKE) -C backend check

engine:
	$(MAKE) -C engine check

check: backend engine

clean:
	$(MAKE) -C backend clean
	$(MAKE) -C engine clean
