# Watcher Crate

Configuration monitoring system with hot-reloading and intelligent server change detection.

## Table of Contents

- [Architecture](docs/architecture.md) - Monitoring system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and event flow
- [Errors](docs/errors.md) - Error types documentation
- [Change Detection](docs/change-detection.md) - Configuration modification detection algorithms
- [Hot-reload](docs/hot-reload.md) - Hot-reload mechanism

## Overview

The `watcher` crate provides a configuration file monitoring system with automatic reload and granular change detection.

- **Real-time monitoring**: Uses `notify` to detect configuration file modifications
- **Intelligent debouncing**: Avoids multiple reloads during repeated saves
- **Change detection**: Precisely identifies added, modified, or removed servers
- **Automatic rescan**: Automatically triggers rescan of modified servers
- **Automatic initialization**: Creates necessary folders for new servers
- **Thread-safe**: Uses RwLock and Arc for concurrency

## Architecture

The system is organized around one main component:

### ConfigWatcher
Monitors the configuration file and reacts to changes. It maintains a shared reference to the active configuration and coordinates with the CacheManager to synchronize changes.

### Granular detection
The watcher compares the old and new configuration to detect:
- Added servers: New servers in the configuration
- Modified servers: Existing servers with significant configuration changes
- Removed servers: Servers removed from the configuration

### Rescan mechanism
Uses a pause/resume system to avoid race conditions during configuration reload. The rescan is paused during the update then automatically resumed.

## Integration

This crate integrates with:
- `lighty_config`: To load and validate the new configuration
- `lighty_cache`: To trigger rescans and manage the cache
- `lighty_filesystem`: To create folder structures for new servers
- `notify`: For system-level file monitoring
