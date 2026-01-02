# Event System Architecture

## Components

```mermaid
graph TB
    EventBus[EventBus]
    AppEvent[AppEvent Enum]
    Config[Config Loader]
    Cache[Cache Manager]
    Scanner[Scanner]
    Watcher[Config Watcher]
    API[API Layer]
    Console[Console Output]
    Logs[Tracing Logs]

    Config --> EventBus
    Cache --> EventBus
    Scanner --> EventBus
    Watcher --> EventBus
    API --> EventBus

    EventBus --> AppEvent
    AppEvent --> Console
    AppEvent --> Logs
```

## Simplified Publisher-Subscriber Pattern

The system uses a simplified pattern without multiple subscribers:
- EventBus centralizes emission
- Synchronous emission to console
- No asynchronous queue
- No event persistence

## Emission Flow

```mermaid
sequenceDiagram
    participant Component
    participant EventBus
    participant Formatter
    participant Console

    Component->>EventBus: emit(AppEvent::Ready)
    EventBus->>Formatter: Format event
    Formatter->>Console: Colored output
```
