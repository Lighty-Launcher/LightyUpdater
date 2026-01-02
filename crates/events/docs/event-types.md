# Event Types

## Lifecycle Events

### Starting
Emitted at application startup.
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  LightyUpdater - Distribution Server
  Version 0.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Ready
Server ready and listening on specified address.
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Server 0.0.0.0:8080
  URL    http://localhost:8080
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Shutdown
Server shutdown.

## Configuration Events

### ConfigLoaded
```
  ✓ 3 server(s)
```

### CacheUpdated
```
  ↻ Updated server1 (mods, libraries)
```

### NewServerDetected
```
  + New server: server2
```

## Error Event

Format: Passed directly to tracing::error
```rust
tracing::error!("{}: {}", context, error);
```
