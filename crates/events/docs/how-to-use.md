# Using lighty-events

Two patterns: emit events as a service crate, or subscribe as a sink.

## 1. Emit from a service crate

Take an `Arc<EventBus>` as a constructor argument, store it, call
`emit` whenever something interesting happens.

```rust
use lighty_events::{AppEvent, EventBus};
use std::sync::Arc;

pub struct MyScanner {
    events: Arc<EventBus>,
}

impl MyScanner {
    pub fn new(events: Arc<EventBus>) -> Self {
        Self { events }
    }

    pub fn scan(&self, server: &str) {
        self.events.emit(AppEvent::ScanStarted {
            server: server.to_string(),
        });
        // ... work ...
        self.events.emit(AppEvent::ScanCompleted {
            server: server.to_string(),
            duration: std::time::Duration::from_secs(1),
        });
    }
}
```

Never construct your own `EventBus`. The composition root
(`lighty-runtime`) builds the bus and hands an `Arc` down.

## 2. Implement a sink

A sink decides what to do with an event. Implementations stay
trivial — keep heavy work behind `tokio::spawn`.

```rust
use lighty_events::{AppEvent, EventSink};

pub struct WebsocketSink {
    tx: tokio::sync::broadcast::Sender<String>,
}

impl EventSink for WebsocketSink {
    fn handle(&self, event: &AppEvent) {
        if let Ok(json) = serde_json::to_string(event) {
            let _ = self.tx.send(json);
        }
    }
}
```

`AppEvent` derives `Serialize`, so JSON-emitting sinks are one
`serde_json::to_string` away.

## 3. Mount multiple sinks

`EventBus::with_sink` only accepts a single `Arc<dyn EventSink>`. To
attach more than one, wrap them in `CompositeEventSink`:

```rust
use lighty_events::{CompositeEventSink, EventBus, EventSink};
use std::sync::Arc;

let sinks: Vec<Arc<dyn EventSink>> = vec![
    Arc::new(ConsoleEventSink::new()),
    Arc::new(WebsocketSink::new(tx)),
];

let bus = EventBus::with_sink(
    /* verbose */ true,
    Arc::new(CompositeEventSink::new(sinks)),
);
```

The fan-out order is the order of the `Vec` — useful when one sink
should observe the event before another.

## 4. Silent / no-op bus

For tests where you don't care about the output:

```rust
let bus = EventBus::new(false); // verbose = false → emit is a no-op
```

Or build a capturing sink for assertions:

```rust
use std::sync::Mutex;

struct CapturingSink {
    seen: Mutex<Vec<AppEvent>>,
}

impl EventSink for CapturingSink {
    fn handle(&self, event: &AppEvent) {
        self.seen.lock().unwrap().push(event.clone());
    }
}

let capture = Arc::new(CapturingSink { seen: Mutex::new(vec![]) });
let bus     = EventBus::with_sink(true, capture.clone());

// ... exercise code under test ...
assert!(capture.seen.lock().unwrap().iter().any(|event| {
    matches!(event, AppEvent::CacheNew { .. })
}));
```

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md) — every `AppEvent` variant
- [`../../cdn/docs/how-to-use.md`](../../cdn/docs/how-to-use.md) —
  example of an async sink that `spawn`s from `handle`
