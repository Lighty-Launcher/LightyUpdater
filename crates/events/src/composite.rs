use super::bus::EventSink;
use super::models::AppEvent;
use std::sync::Arc;

// Fan-out sink that forwards every event to each child in order.
pub struct CompositeEventSink {
    sinks: Vec<Arc<dyn EventSink>>,
}

impl CompositeEventSink {
    pub fn new(sinks: Vec<Arc<dyn EventSink>>) -> Self {
        Self { sinks }
    }
}

impl EventSink for CompositeEventSink {
    fn handle(&self, event: &AppEvent) {
        for sink in &self.sinks {
            sink.handle(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct CountingSink {
        count: Mutex<usize>,
    }

    impl CountingSink {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                count: Mutex::new(0),
            })
        }

        fn get(&self) -> usize {
            *self.count.lock().unwrap()
        }
    }

    impl EventSink for CountingSink {
        fn handle(&self, _event: &AppEvent) {
            *self.count.lock().unwrap() += 1;
        }
    }

    #[test]
    fn fans_out_to_every_child_sink() {
        let a = CountingSink::new();
        let b = CountingSink::new();
        let composite = CompositeEventSink::new(vec![a.clone(), b.clone()]);

        composite.handle(&AppEvent::Shutdown);
        composite.handle(&AppEvent::Shutdown);

        assert_eq!(a.get(), 2);
        assert_eq!(b.get(), 2);
    }

    #[test]
    fn empty_composite_is_noop() {
        let composite = CompositeEventSink::new(vec![]);
        composite.handle(&AppEvent::Shutdown);
    }
}
