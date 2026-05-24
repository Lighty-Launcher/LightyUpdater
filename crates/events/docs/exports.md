# Exports

Public surface of `lighty-events`.

## Crate root

```rust
use lighty_events::{AppEvent, CompositeEventSink, EventBus, EventSink};
```

## `EventBus`

```rust
pub struct EventBus { /* verbose: bool, sink: Option<Arc<dyn EventSink>> */ }

impl EventBus {
    pub fn new(verbose: bool) -> Arc<Self>;
    pub fn with_sink(verbose: bool, sink: Arc<dyn EventSink>) -> Arc<Self>;
    pub fn emit(&self, event: AppEvent);
}
```

`emit` short-circuits when `verbose == false`. The bus holds exactly
one sink; use `CompositeEventSink` for fan-out.

## `EventSink`

```rust
pub trait EventSink: Send + Sync {
    fn handle(&self, event: &AppEvent);
}
```

Synchronous: implementations that need async work must `spawn`
internally.

## `CompositeEventSink`

```rust
pub struct CompositeEventSink { /* sinks: Vec<Arc<dyn EventSink>> */ }

impl CompositeEventSink {
    pub fn new(sinks: Vec<Arc<dyn EventSink>>) -> Self;
}

impl EventSink for CompositeEventSink {
    fn handle(&self, event: &AppEvent);
}
```

## `AppEvent`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    // Application lifecycle
    Starting { version: String },
    Ready    { addr: String, base_url: String },
    Shutdown,

    // Configuration
    ConfigLoading { path: String },
    ConfigLoaded  { servers_count: usize },
    ConfigCreated { path: String },
    ConfigMigrated { added_fields: Vec<String> },
    ConfigReloaded,
    ConfigError   { error: String },

    // Server initialization
    ServerFolderInit    { name: String, path: String },
    ServerFolderCreated { name: String },
    AllServersInitialized,

    // Scanning
    ScanStarted        { server: String },
    ScanCompleted      { server: String, duration: Duration },
    InitialScanStarted,

    // Cache lifecycle
    CacheNew       { server: String },
    CacheUpdated   { server: String, changes: Vec<String> },
    CacheUnchanged { server: String },

    // Server discovery (from the watcher)
    NewServerDetected { name: String },
    ServerRemoved     { name: String },

    // Auto-scan / file watcher modes
    AutoScanEnabled      { interval: u64 },
    ContinuousScanEnabled,

    // CDN / edge cache
    CdnPurgeRequested        { server: String, urls: Vec<String> },
    CdnPurgeCompleted        { server: String, ok: bool, error: Option<String> },
    CloudflarePurgeRequested { server: String },
    CloudflarePurgeCompleted { server: String, ok: bool, error: Option<String> },

    // Errors
    Error { context: String, error: String },
}
```

### Variant ownership

| Variant | Emitted by |
|---|---|
| `Starting`, `Ready`, `Shutdown` | `lighty-runtime::run_server` |
| `ConfigLoading`, `ConfigLoaded`, `ConfigCreated`, `ConfigMigrated`, `ConfigReloaded`, `ConfigError` | `lighty-adapters::config_io_loader`, `lighty-watcher` |
| `ServerFolderInit`, `ServerFolderCreated`, `AllServersInitialized` | `lighty-runtime::bootstrap::initialize_folders` |
| `ScanStarted`, `ScanCompleted` | `lighty-scanner` (loud variant only) |
| `InitialScanStarted` | `lighty-cache::CacheManager::initialize` |
| `CacheNew`, `CacheUpdated`, `CacheUnchanged` | `lighty-rescan` |
| `NewServerDetected`, `ServerRemoved` | `lighty-watcher` |
| `AutoScanEnabled`, `ContinuousScanEnabled` | `lighty-rescan::run_rescan_loop` |
| `CdnPurgeRequested`, `CloudflarePurgeRequested` | `lighty-rescan::update_cache_if_changed` |
| `CdnPurgeCompleted`, `CloudflarePurgeCompleted` | reserved — `lighty-cdn::CdnEventSink` may emit these once the bus gains a back-edge |
| `Error` | anywhere a recoverable error needs operator visibility |

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`../../rescan/docs/events.md`](../../rescan/docs/events.md) — full
  catalogue of rescan-side emission sites
- [`../../adapters/docs/overview.md`](../../adapters/docs/overview.md) —
  the `ConsoleEventSink` implementation
