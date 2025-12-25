# Events Crate

Event bus system for inter-component communication with structured console display.

## Table of Contents

- [Architecture](docs/architecture.md) - Event system architecture
- [Event Types](docs/event-types.md) - Complete event catalog
- [Formatting](docs/formatting.md) - Console display system

## Overview

The `events` crate provides a centralized event bus system for inter-component communication with formatted console display.

- **EventBus**: Centralized bus for event emission
- **AppEvent**: Enumeration of all possible event types
- **Structured display**: Console messages with colors and symbols
- **Silent mode**: Option to disable display
- **Categorization**: Events grouped by context

## Event Types

### Lifecycle
- Starting: Application startup
- Ready: Server ready with address and URL
- Shutdown: Server shutdown

### Configuration
- ConfigLoading: Configuration loading
- ConfigLoaded: Configuration loaded successfully
- ConfigCreated: Default configuration created
- ConfigMigrated: Fields added during migration
- ConfigReloaded: Configuration hot-reloaded
- ConfigError: Configuration error

### Scanning
- ScanStarted: Server scan started
- ScanCompleted: Scan completed with duration
- InitialScanStarted: Initial scan of all servers

### Cache
- CacheNew: New cache created for a server
- CacheUpdated: Cache updated with change list
- CacheUnchanged: No change detected

### Server Discovery
- NewServerDetected: New server added to config
- ServerRemoved: Server removed from config

### Auto-scan
- AutoScanEnabled: Automatic rescan enabled with interval
- ContinuousScanEnabled: Continuous monitoring enabled

### Errors
- Error: General error with context and message

## Integration

This crate integrates with:
- All project crates for event emission
- `colored`: For colored console display
- `tracing`: For structured logs
