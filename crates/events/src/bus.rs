use super::models::AppEvent;
use std::sync::Arc;

pub trait EventSink: Send + Sync {
    fn handle(&self, event: &AppEvent);
}

pub struct EventBus {
    // Legacy semantic: historically `true` meant event output enabled.
    silent_mode: bool,
    sink: Option<Arc<dyn EventSink>>,
}

impl EventBus {
    pub fn new(silent_mode: bool) -> Arc<Self> {
        Arc::new(Self {
            silent_mode,
            sink: None,
        })
    }

    pub fn with_sink(silent_mode: bool, sink: Arc<dyn EventSink>) -> Arc<Self> {
        Arc::new(Self {
            silent_mode,
            sink: Some(sink),
        })
    }

    pub fn emit(&self, event: AppEvent) {
        if !self.silent_mode {
            return;
        }

        if let Some(sink) = &self.sink {
            sink.handle(&event);
        }
    }
}
