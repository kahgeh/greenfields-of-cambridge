# E2E Tests with Playwright

This directory contains end-to-end tests for the Greenfields of Cambridge website using Playwright.

## Setup

1. Install Node.js dependencies:
```bash
npm install
```

2. Install Playwright browsers:
```bash
npx playwright install
```

## Running Tests

### Using Make (Recommended)

The easiest way to run E2E tests is with the Makefile:

```bash
# Run all E2E tests (automatically starts server)
make test-e2e

# Run with headed browser for debugging
make test-e2e-debug
```

This will:
1. Start the Rust server on the default port (7100)
2. Wait for it to be ready
3. Run all Playwright tests
4. Shut down the server

### Manual Execution

You can also run tests manually:

```bash
# Start the server in one terminal
cargo run

# In another terminal, run tests
npm test

# Run with headed browser for debugging
npm run test:headed

# Run in debug mode with Playwright Inspector
npm run test:debug
```

## Writing Tests

Tests should be written in TypeScript and placed in this directory with the `.spec.ts` extension.

Example test structure:

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test('does something', async ({ page }) => {
    await page.goto('/');
    // Test logic here
  });
});
```

## Test Reports

After running tests, an HTML report will be generated in the `playwright-report` directory. Open it with:

```bash
npx playwright show-report
```

## Environment Variables

- `TEST_SERVER_URL`: Base URL of the test server (set automatically by Rust harness)
- `CI`: Set to true in CI environments to disable headed mode