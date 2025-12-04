# E2E Testing Guide

This guide explains how to run and maintain the E2E tests for the Greenfields of Cambridge website.

## Overview

The E2E tests use Playwright (TypeScript/JavaScript) to test the web application, with Rust tooling orchestrating server startup/shutdown via the Makefile.

## Quick Start

### For Automated Validation (CI/CD, git hooks, etc.)

```bash
# Run tests with console output - no hanging processes
make test-e2e
```

- Starts server on port 7100
- Runs tests with list reporter (shows detailed test results in console)
- Stops server
- Clean exit with proper status code
- No hanging report servers

Example output:
```
Running 6 tests using 6 workers

  ✓ [chromium] › home-page.spec.ts:3:1 › Home Page » loads successfully (1.2s)
  ✓ [chromium] › home-page.spec.ts:9:1 › Home Page » has navigation to contact page (0.8s)
  ✓ [chromium] › contact-form.spec.ts:3:1 › Contact Form » displays contact form (0.5s)
  ✓ [chromium] › contact-form.spec.ts:14:1 › Contact Form » shows validation errors (0.6s)
  ✓ [chromium] › contact-form.spec.ts:24:1 › Contact Form » submits form successfully (2.1s)
  ✓ [chromium] › contact-form.spec.ts:40:1 › Contact Form » binds form inputs (0.4s)

  6 passed (5.6s)
```

### For Local Development

```bash
# Run tests with HTML report
make test-e2e-report

# View the report (after tests complete)
cd tests/playwright
npx playwright show-report

# Or see test details in console with color
npx playwright test --reporter=list
```

```bash
# Debug with visible browser
make test-e2e-debug
```

## Manual Testing (if needed)

```bash
# Terminal 1: Start server
cargo run

# Terminal 2: Run tests
cd tests/playwright
npm test
```

## Project Structure

```
tests/playwright/
├── package.json           # Node.js dependencies
├── playwright.config.ts   # Playwright configuration
├── tsconfig.json         # TypeScript configuration
├── README.md             # Playwright-specific docs
└── *.spec.ts             # Test files
```

## Writing Tests

1. Create test files in `tests/playwright/` with `.spec.ts` extension
2. Use TypeScript for type safety
3. Test example:

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature', () => {
  test('does something', async ({ page }) => {
    await page.goto('/');
    // Test logic
  });
});
```

## Important Notes

- **Default server URL**: `http://localhost:7100`
- **Server management**: Handled by Makefile, not Playwright
- **Report servers**: Only `test-e2e-report` generates HTML (but doesn't auto-serve)
- **Dependencies**: Auto-installed on first run via `make install-deps`

## Troubleshooting

### Port already in use
```bash
# Kill any process on port 7100
lsof -ti:7100 | xargs kill -9
```

### Clean everything
```bash
make clean
rm -rf tests/playwright/node_modules
rm -rf tests/playwright/playwright-report
```

### Dependencies issues
```bash
make install-deps
```

## Reporter Options

Playwright has several reporters for different use cases:

- `list` (default in `make test-e2e`): Detailed output with colors, shows each test name and result
- `line`: Compact one-line per test
- `json`: Machine-readable JSON output
- `html`: Visual HTML report (used in `make test-e2e-report`)
- `junit`: JUnit XML format for CI systems
- `dot`: Simple dot progress indicator

To run tests sequentially (one at a time) for better readability:
```bash
cd tests/playwright && npx playwright test --workers=1
```

To combine reporters:
```bash
cd tests/playwright && npx playwright test --reporter=list,html
```

## CI/CD Integration

Use `make test-e2e` for automated testing. It:
- Returns proper exit codes
- Uses console output (human-readable but parseable)
- Leaves no background processes

For CI systems that prefer JUnit:
```bash
cd tests/playwright && npx playwright test --reporter=junit --outputFile=test-results.xml
```