# Events Crate

Event contracts and bus abstraction for LightyUpdater.

## Responsibilities

- Define `AppEvent` as the shared event model.
- Expose `EventSink` abstraction for pluggable renderers.
- Expose `EventBus` for event dispatch to sink implementations.

## Non-Responsibilities

- No console formatting.
- No terminal coloring.
- No transport-specific rendering.

## Related Crate

- `lighty-events-console` provides the console sink implementation.
