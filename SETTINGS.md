# Settings Configuration

The greenfields-of-cambridge project uses a hierarchical configuration system based on the `config` crate.

## Configuration Files

Configuration files are located in the `config/` directory:

- `default.toml` - Base configuration values (required)
- `local.toml` - Local development overrides (optional)
- `{environment}.toml` - Environment-specific overrides (optional, e.g., `production.toml`, `staging.toml`)

## Environment Variables

Settings can be overridden using environment variables with the prefix `app`. Use double underscores (`__`) to separate nested keys.

Examples:
```bash
# Override server port
export APP_SERVER__PORT=8080

# Override log level
export APP_LOG__LEVEL=info

# Override log format
export APP_LOG__FORMAT=json
```

## Loading Priority

1. Default values (hardcoded)
2. `config/default.toml`
3. `config/{environment}.toml` (where environment comes from `run_environment` env var)
4. Environment variables (prefix: `app`)

## Running with Different Environments

```bash
# Local development (default)
cargo run

# Production
run_environment=production cargo run

# Staging
run_environment=staging cargo run
```

## Configuration Options

### Server
- `host`: Server bind address (default: "0.0.0.0")
- `port`: Server port (default: 7100)

### Logging
- `level`: Log level (debug, info, warn, error)
- `format`: Log format (text, json)

### Metadata
- `name`: Application name (from Cargo.toml)
- `version`: Application version (from Cargo.toml)