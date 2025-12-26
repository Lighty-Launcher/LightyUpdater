# Cache Crate

Intelligent cache system for managing Minecraft server versions with granular change detection and cloud synchronization.

## Table of Contents

- [Architecture](docs/architecture.md) - Global cache system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and data flows
- [Errors](docs/errors.md) - Error types documentation
- [Rescan Orchestrator](docs/rescan.md) - Automatic and manual rescan system
- [FileDiff](docs/file-diff.md) - Granular change detection
- [Cache Manager](docs/manager.md) - Main cache manager

## Integration

This crate integrates with:
- `lighty_scanner`: To scan server structures
- `lighty_storage`: For persistence and cloud synchronization
- `lighty_config`: For cache and server configuration
- `lighty_events`: To emit events on cache changes
- `lighty_models`: For `VersionBuilder` data structures
