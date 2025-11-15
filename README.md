# rs-config-loader

[![CI](https://github.com/philiprehberger/rs-config-loader/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-config-loader/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-config-loader.svg)](https://crates.io/crates/philiprehberger-config-loader)
[![GitHub release](https://img.shields.io/github/v/release/philiprehberger/rs-config-loader)](https://github.com/philiprehberger/rs-config-loader/releases)
[![Last updated](https://img.shields.io/github/last-commit/philiprehberger/rs-config-loader)](https://github.com/philiprehberger/rs-config-loader/commits/main)
[![License](https://img.shields.io/github/license/philiprehberger/rs-config-loader)](LICENSE)
[![Bug Reports](https://img.shields.io/github/issues/philiprehberger/rs-config-loader/bug)](https://github.com/philiprehberger/rs-config-loader/issues?q=is%3Aissue+is%3Aopen+label%3Abug)
[![Feature Requests](https://img.shields.io/github/issues/philiprehberger/rs-config-loader/enhancement)](https://github.com/philiprehberger/rs-config-loader/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)
[![Sponsor](https://img.shields.io/badge/sponsor-GitHub%20Sponsors-ec6cb9)](https://github.com/sponsors/philiprehberger)

Layered configuration from files and environment variables

## Installation

```toml
[dependencies]
philiprehberger-config-loader = "0.1.9"
```

## Usage

```rust
use philiprehberger_config_loader::{ConfigBuilder, ConfigValue};

let config = ConfigBuilder::new()
    .default("port", 8080_i64)
    .default("debug", false)
    .add_file("config.toml")
    .add_env_prefix("APP")
    .set("version", "1.0.0")
    .build()
    .unwrap();

// Typed getters
let port = config.get_int("port");         // Some(8080)
let debug = config.get_bool("debug");      // Some(false)
let ver = config.get_string("version");    // Some("1.0.0")
```

### Configuration file (TOML subset)

```toml
# config.toml
host = "localhost"
port = 3000
debug = true

[database]
url = "postgres://localhost/mydb"
pool_size = 5
```

### Environment variables

With prefix `APP`, environment variables map as follows:

| Environment Variable | Config Key |
|---------------------|------------|
| `APP_PORT` | `port` |
| `APP_DATABASE__URL` | `database.url` |

Double underscore (`__`) maps to dot-separated nesting.

### Layer priority

Later layers override earlier ones:

1. Defaults (lowest)
2. File values
3. Environment variables
4. Manual overrides (highest)

## API

| Type | Description |
|------|-------------|
| `ConfigBuilder` | Builder for assembling configuration layers |
| `Config` | Immutable configuration store |
| `ConfigValue` | Enum: `String`, `Integer`, `Float`, `Bool`, `Array` |
| `ConfigError` | Error: `FileNotFound`, `ParseError`, `TypeError` |

### ConfigBuilder

| Method | Description |
|--------|-------------|
| `ConfigBuilder::new()` | Create a new builder |
| `.default(key, value)` | Set a default value |
| `.add_file(path)` | Add a TOML file source |
| `.add_env_prefix(prefix)` | Add env var source with prefix |
| `.set(key, value)` | Manual override (highest priority) |
| `.build()` | Build the `Config` |

### Config

| Method | Description |
|--------|-------------|
| `.get(key)` | Get a `&ConfigValue` |
| `.get_string(key)` | Get as `&str` |
| `.get_int(key)` | Get as `i64` |
| `.get_float(key)` | Get as `f64` |
| `.get_bool(key)` | Get as `bool` |
| `.keys()` | Iterate over all keys |


## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Support

If you find this package useful, consider giving it a star on GitHub — it helps motivate continued maintenance and development.

[![LinkedIn](https://img.shields.io/badge/Philip%20Rehberger-LinkedIn-0A66C2?logo=linkedin)](https://www.linkedin.com/in/philiprehberger)
[![More packages](https://img.shields.io/badge/more-open%20source%20packages-blue)](https://philiprehberger.com/open-source-packages)

## License

[MIT](LICENSE)
