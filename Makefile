.PHONY: help frontend backend engine check clean

help:
	@echo "Available commands:"
	@echo ""
	@echo "  make frontend  Run frontend checks"
	@echo "  make backend   Run backend checks"
	@echo "  make engine    Run engine checks"
	@echo "  make check     Run all checks"
	@echo "  make clean     Clean backend and engine artifacts"

frontend:
	$(MAKE) -C frontend check

backend:
	$(MAKE) -C backend check

engine:
	$(MAKE) -C engine check

check: frontend backend engine

clean:
	$(MAKE) -C frontend clean
	$(MAKE) -C backend clean
	$(MAKE) -C engine clean
