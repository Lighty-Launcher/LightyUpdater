# Cache Crate

Intelligent cache system for managing Minecraft server versions with granular change detection and cloud synchronization.

## Table of Contents

- [Architecture](docs/architecture.md) - Global cache system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and data flows
- [Errors](docs/errors.md) - Error types documentation
- [Rescan Orchestrator](docs/rescan.md) - Automatic and manual rescan system
- [FileDiff](docs/file-diff.md) - Granular change detection
- [Cache Manager](docs/manager.md) - Main cache manager

## Overview

The `cache` crate provides a multi-layer cache system for efficient management of Minecraft server versions. It includes:

- **Granular change detection**: The `FileDiff` system precisely detects added, modified, or removed files between two versions
- **Automatic rescanning**: `RescanOrchestrator` monitors servers and updates the cache automatically via polling or file watcher
- **Cloud synchronization**: Integration with remote storage backends (S3, etc.) to synchronize modified files
- **In-memory cache**: Uses Moka for performant LRU caching of frequently accessed files
- **Optimized lookup**: `ServerPathCache` provides O(1) lookups to determine which server a file belongs to

## Architecture

The system is organized around several key components:

### CacheManager
The main manager that coordinates all cache components. It maintains a DashMap cache of `VersionBuilder` for each server and manages the lifecycle of background tasks.

### RescanOrchestrator
Orchestrates automatic and manual server rescanning. It supports two modes:
- **Polling mode**: Periodic rescan based on a configurable interval
- **File watcher mode**: Real-time monitoring of file changes with debouncing

### FileDiff
Computes granular differences between two server versions, identifying exactly which files changed (client, libraries, mods, natives, assets). Optimized for performance with HashMaps for O(1) lookups.

### ServerPathCache
Fast mapping cache between file paths and servers. Essential for the file watcher to determine which server to rescan when a file changes.

### FileCacheManager
Manages in-memory file cache with Moka LRU. Reduces latency for frequently accessed files and optimizes memory usage.

## Integration

This crate integrates with:
- `lighty_scanner`: To scan server structures
- `lighty_storage`: For persistence and cloud synchronization
- `lighty_config`: For cache and server configuration
- `lighty_events`: To emit events on cache changes
- `lighty_models`: For `VersionBuilder` data structures
