# Using lighty-adapters

Two unrelated things in this crate; pick the section you need.

## 1. Load the config

```rust
use lighty_adapters::{load_config, ConsoleEventSink};
use lighty_events::EventBus;
use std::sync::Arc;

let bus    = EventBus::with_sink(true, Arc::new(ConsoleEventSink::new()));
let config = load_config("config.toml", &bus).await?;
```

What happens:
- File missing → write defaults, emit `ConfigCreated`.
- File present → parse, run `migrate_config_if_needed`, emit
  `ConfigLoading` then `ConfigMigrated { added_fields }` if anything
  was added.
- Validate, emit `ConfigLoaded { servers_count }`.

## 2. Mount the console sink on a bus

```rust
use lighty_adapters::ConsoleEventSink;
use lighty_events::EventBus;
use std::sync::Arc;

let bus = EventBus::with_sink(true, Arc::new(ConsoleEventSink::new()));
```

`ConsoleEventSink` is stateless — construct it via `::new()` or
`Default::default()` and clone the `Arc` as many times as you need.

## 3. Pair console with other sinks (fan-out)

```rust
use lighty_adapters::ConsoleEventSink;
use lighty_cdn::CdnEventSink;
use lighty_events::{CompositeEventSink, EventBus, EventSink};
use std::sync::Arc;

let sinks: Vec<Arc<dyn EventSink>> = vec![
    Arc::new(ConsoleEventSink::new()),
    Arc::new(CdnEventSink::new(cdn, cloudflare)),
];
let bus = EventBus::with_sink(true, Arc::new(CompositeEventSink::new(sinks)));
```

This is the canonical setup `lighty-runtime` uses.

## Errors at a glance

```rust
pub enum AdaptersError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Validation(String),
}
```

Surfaced on `RuntimeError` via `#[from]`.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`../../config/docs/overview.md`](../../config/docs/overview.md) —
  the schema this crate parses
