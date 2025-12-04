# Greenfields of Cambridge

A web application for Greenfields of Cambridge lawn care services, built with Rust and Datastar hypermedia.

## Quick Start

### Build and Run

```bash
# Build the project
make build

# Run the server
make run
# or
cargo run
```

The server will be available at `http://localhost:7100`

## Testing

### Unit Tests

```bash
make test
# or
cargo test
```

### E2E Tests with Playwright

For automated validation (CI/CD, git hooks, etc.):
```bash
make test-e2e
```

For local development with HTML report:
```bash
make test-e2e-report
```

For debugging with visible browser:
```bash
make test-e2e-debug
```

See [docs/e2e-testing-guide.md](docs/e2e-testing-guide.md) for detailed E2E testing instructions.

## Project Structure

```
src/
├── handlers/          # HTTP handlers
│   ├── mod.rs
│   ├── pages.rs       # Page handlers
│   └── contact.rs     # Contact form handlers
├── main.rs            # Server setup
├── lib.rs
├── error.rs
└── settings.rs

tests/playwright/      # E2E tests
└── *.spec.ts

templates/             # HTML templates
```

## Dependencies

- Rust toolchain
- Node.js (for E2E tests)
- Make

## Development

1. Install Rust dependencies: `cargo build`
2. Install Playwright dependencies: `make install-deps`
3. Run tests: `make test` (unit) or `make test-e2e` (E2E)