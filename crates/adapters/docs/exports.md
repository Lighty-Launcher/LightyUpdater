# Exports

Public surface of `lighty-adapters`.

## Crate root

```rust
use lighty_adapters::{load_config, AdaptersError, ConsoleEventSink};
```

## `load_config`

```rust
pub async fn load_config(
    path: impl AsRef<Path>,
    events: &Arc<lighty_events::EventBus>,
) -> Result<lighty_config::Config, AdaptersError>;
```

Reads (or creates) the TOML file at `path`, applies the migration
pipeline, validates, and returns the hydrated `Config`. Emits one or
more of `ConfigLoading`, `ConfigCreated`, `ConfigMigrated`,
`ConfigLoaded`, `ConfigError` along the way.

## `ConsoleEventSink`

```rust
pub struct ConsoleEventSink;

impl ConsoleEventSink {
    pub fn new() -> Self;
}

impl Default for ConsoleEventSink { /* delegates to new */ }
impl lighty_events::EventSink for ConsoleEventSink { /* match every AppEvent */ }
```

Stateless; can be wrapped in an `Arc` once and shared.

## `AdaptersError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum AdaptersError {
    #[error("I/O error: {0}")] Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")] Toml(#[from] toml::de::Error),
    #[error("Validation error: {0}")] Validation(String),
}
```

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`../../events/docs/exports.md`](../../events/docs/exports.md)
