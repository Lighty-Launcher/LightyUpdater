use super::models::AppEvent;
use std::sync::Arc;

pub trait EventSink: Send + Sync {
    fn handle(&self, event: &AppEvent);
}

// When `verbose` is true the bus dispatches every emitted event to its sink.
// When false the bus is silent and `emit` is a no-op.
pub struct EventBus {
    verbose: bool,
    sink: Option<Arc<dyn EventSink>>,
}

impl EventBus {
    pub fn new(verbose: bool) -> Arc<Self> {
        Arc::new(Self {
            verbose,
            sink: None,
        })
    }

    pub fn with_sink(verbose: bool, sink: Arc<dyn EventSink>) -> Arc<Self> {
        Arc::new(Self {
            verbose,
            sink: Some(sink),
        })
    }

    pub fn emit(&self, event: AppEvent) {
        if !self.verbose {
            return;
        }

        if let Some(sink) = &self.sink {
            sink.handle(&event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct CapturingSink {
        events: Mutex<Vec<AppEvent>>,
    }

    impl CapturingSink {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                events: Mutex::new(Vec::new()),
            })
        }

        fn snapshot(&self) -> Vec<AppEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    impl EventSink for CapturingSink {
        fn handle(&self, event: &AppEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    #[test]
    fn emit_dispatches_to_sink_when_verbose() {
        let sink = CapturingSink::new();
        let bus = EventBus::with_sink(true, sink.clone());

        bus.emit(AppEvent::Shutdown);

        assert_eq!(sink.snapshot().len(), 1);
    }

    #[test]
    fn emit_is_noop_when_not_verbose() {
        let sink = CapturingSink::new();
        let bus = EventBus::with_sink(false, sink.clone());

        bus.emit(AppEvent::Shutdown);

        assert!(sink.snapshot().is_empty());
    }

    #[test]
    fn emit_without_sink_is_noop() {
        let bus = EventBus::new(true);
        bus.emit(AppEvent::Shutdown);
    }
}
