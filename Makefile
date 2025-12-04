.PHONY: help build run test test-e2e test-e2e-report test-e2e-debug install-deps clean

# Default target
help:
	@echo "Available commands:"
	@echo "  build          - Build the project"
	@echo "  run            - Run the server"
	@echo "  test           - Run unit tests"
	@echo "  test-e2e       - Run E2E tests with detailed console output (no hanging report server)"
	@echo "  test-e2e-report - Run E2E tests with HTML report (for local development)"
	@echo "  test-e2e-debug - Run E2E tests with headed browser for debugging"
	@echo "  install-deps   - Install Playwright dependencies"
	@echo "  clean          - Clean build artifacts"

# Build the project
build:
	cargo build --release

# Run the server
run:
	cargo run

# Run unit tests
test:
	cargo test

# Install Playwright and npm dependencies
install-deps:
	@echo "Installing npm dependencies..."
	cd tests/playwright && npm install
	@echo "Installing Playwright browsers..."
	cd tests/playwright && npx playwright install

# Run E2E tests (this will start the server, run tests, and shut down)
test-e2e: install-deps
	@echo "Starting E2E tests..."
	@# Start the server in the background
	@cargo run > /dev/null 2>&1 & echo $$! > .server.pid
	@# Wait for server to be ready with health check
	@echo "Waiting for server to start..."
	@for i in $$(seq 1 30); do \
		if curl -s http://localhost:7100 > /dev/null 2>&1; then \
			echo "Server is ready!"; \
			break; \
		fi; \
		sleep 1; \
	done
	@# Run the tests with console reporter (no report server)
	@cd tests/playwright && npx playwright test --reporter=list; \
	EXIT_CODE=$$?; \
	# Stop the server
	@echo "Stopping server..."
	@if [ -f .server.pid ]; then \
		kill -TERM `cat .server.pid` 2>/dev/null || true; \
		sleep 1; \
		kill -KILL `cat .server.pid` 2>/dev/null || true; \
		rm -f .server.pid; \
	fi
	@# Exit with the test exit code
	@exit $$EXIT_CODE

# Run E2E tests with HTML report (for local development)
test-e2e-report: install-deps
	@echo "Starting E2E tests with HTML report..."
	@# Start the server in the background
	@cargo run > /dev/null 2>&1 & echo $$! > .server.pid
	@# Wait for server to be ready with health check
	@echo "Waiting for server to start..."
	@for i in $$(seq 1 30); do \
		if curl -s http://localhost:7100 > /dev/null 2>&1; then \
			echo "Server is ready!"; \
			break; \
		fi; \
		sleep 1; \
	done
	@# Run the tests with HTML reporter
	@cd tests/playwright && npm test; \
	EXIT_CODE=$$?; \
	# Stop the server
	@echo "Stopping server..."
	@if [ -f .server.pid ]; then \
		kill -TERM `cat .server.pid` 2>/dev/null || true; \
		sleep 1; \
		kill -KILL `cat .server.pid` 2>/dev/null || true; \
		rm -f .server.pid; \
	fi
	@# Show report if tests passed
	@if [ $$EXIT_CODE -eq 0 ]; then \
		echo "Test completed successfully!"; \
		echo "View HTML report with: cd tests/playwright && npx playwright show-report"; \
	fi
	@# Exit with the test exit code
	@exit $$EXIT_CODE

# Run E2E tests in debug mode (headed browser)
test-e2e-debug: install-deps
	@echo "Starting E2E tests in debug mode..."
	@# Start the server in the background
	@cargo run > /dev/null 2>&1 & echo $$! > .server.pid
	@# Wait for server to be ready with health check
	@echo "Waiting for server to start..."
	@for i in $$(seq 1 30); do \
		if curl -s http://localhost:7100 > /dev/null 2>&1; then \
			echo "Server is ready!"; \
			break; \
		fi; \
		sleep 1; \
	done
	@# Run the tests in debug mode
	@cd tests/playwright && npm run test:debug; \
	EXIT_CODE=$$?; \
	# Stop the server
	@echo "Stopping server..."
	@if [ -f .server.pid ]; then \
		kill -TERM `cat .server.pid` 2>/dev/null || true; \
		sleep 1; \
		kill -KILL `cat .server.pid` 2>/dev/null || true; \
		rm -f .server.pid; \
	fi
	@# Exit with the test exit code
	@exit $$EXIT_CODE

# Clean build artifacts
clean:
	cargo clean
	cd tests/playwright && rm -rf node_modules playwright-report
	@if [ -f .server.pid ]; then \
		kill `cat .server.pid` 2>/dev/null || true; \
		rm .server.pid; \
	fi