# E2E Tests - 3D Print Quote Service

End-to-end tests using Playwright for the complete 3D Print Quote Service.

## Setup

```bash
# Install dependencies
pnpm install

# Install Playwright browsers
pnpm exec playwright install
```

## Running Tests

```bash
# Run all E2E tests
pnpm test:e2e

# Run tests with UI mode (interactive)
pnpm test:e2e:ui

# Run tests in headed mode (see browser)
pnpm test:e2e:headed

# Run tests in debug mode
pnpm test:e2e:debug

# Run only API tests
pnpm test:api

# Run only UI tests
pnpm test:ui
```

## Test Structure

```
tests/
├── api/                    # Backend API tests
│   ├── sessions.spec.js    # Session management
│   ├── materials.spec.js   # Material listing
│   └── quote.spec.js       # Quote generation
├── ui/                     # Frontend UI tests
│   └── homepage.spec.js    # Homepage interactions
├── fixtures/               # Test data and fixtures
└── README.md
```

## Configuration

The Playwright configuration is in `playwright.config.js`. Key features:

- **Auto-start servers**: Backend (Cargo) and Frontend (Python HTTP server)
- **Parallel execution**: Tests run in parallel for speed
- **Retry on failure**: Automatic retries in CI environment
- **Screenshot on failure**: Captures screenshots for debugging
- **HTML reporter**: Generates detailed HTML test report

## Test Categories

### API Tests
- Session creation and validation
- Material listing and filtering
- Model upload and processing
- Quote generation and pricing
- Admin endpoints

### UI Tests
- Page load and elements
- Material selection
- File upload interactions
- 3D viewer display
- Quote summary updates

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Tests clean up their own data
3. **Assertions**: Use specific, descriptive assertions
4. **Timeouts**: Allow reasonable time for async operations
5. **Selectors**: Use stable selectors (ids, data attributes)

## CI Integration

Tests are configured to run in CI with:
- Single worker (to avoid conflicts)
- Automatic retries (2x)
- Trace collection on failure
- Video recording on retry

## Debugging

```bash
# Open Playwright Inspector
pnpm exec playwright test --debug

# Run specific test file
pnpm exec playwright test tests/api/sessions.spec.js

# Show test report
pnpm exec playwright show-report
```

## Notes

- Backend must be compiled before running tests
- Database is reset between test runs (if configured)
- WebGL tests may require additional setup in headless environments
