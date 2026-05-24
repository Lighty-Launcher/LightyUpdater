# Using lighty-events

## Emit from a service crate

```rust
use lighty_events::{AppEvent, EventBus};
use std::sync::Arc;

pub struct MyScanner { events: Arc<EventBus> }

impl MyScanner {
    pub fn scan(&self, server: &str) {
        self.events.emit(AppEvent::ScanStarted { server: server.into() });
        // ... work ...
        self.events.emit(AppEvent::ScanCompleted {
            server: server.into(),
            duration: std::time::Duration::from_secs(1),
        });
    }
}
```

Never construct your own `EventBus` — `lighty-runtime` does that and
hands `Arc<EventBus>` down.

## Implement a sink

```rust
use lighty_events::{AppEvent, EventSink};

pub struct WebsocketSink { tx: tokio::sync::broadcast::Sender<String> }

impl EventSink for WebsocketSink {
    fn handle(&self, event: &AppEvent) {
        if let Ok(json) = serde_json::to_string(event) {
            let _ = self.tx.send(json);
        }
    }
}
```

`AppEvent` derives `Serialize`.

## Mount multiple sinks

```rust
use lighty_events::{CompositeEventSink, EventBus, EventSink};
use std::sync::Arc;

let sinks: Vec<Arc<dyn EventSink>> = vec![
    Arc::new(ConsoleEventSink::new()),
    Arc::new(WebsocketSink::new(tx)),
];
let bus = EventBus::with_sink(true, Arc::new(CompositeEventSink::new(sinks)));
```

## Silent bus for tests

```rust
let bus = EventBus::new(false); // emit is a no-op
```

Capturing sink for assertions:

```rust
use std::sync::Mutex;

struct CapturingSink { seen: Mutex<Vec<AppEvent>> }

impl EventSink for CapturingSink {
    fn handle(&self, event: &AppEvent) {
        self.seen.lock().unwrap().push(event.clone());
    }
}
```

## See also

- [overview.md](./overview.md)
- [exports.md](./exports.md)
