# API Crate

HTTP API layer for Minecraft file distribution with intelligent RAM cache management and optimized streaming.

## Table of Contents

- [Architecture](docs/architecture.md) - HTTP API architecture
- [Processing Flow](docs/flow.md) - Route sequence diagrams
- [Errors](docs/errors.md) - Error types documentation
- [Handlers](docs/handlers.md) - Routes and handlers documentation
- [File Serving](docs/file-serving.md) - File distribution system
- [Resolution](docs/resolution.md) - URL to file path resolution

## Overview

The `api` crate provides the HTTP interface to access Minecraft server files and metadata.

- **Axum Framework**: High-performance, type-safe REST API with Tokio
- **Intelligent serving**: RAM cache for small files, streaming for large files
- **O(1) resolution**: URL to file path mapping via HashMap
- **Security validation**: Protection against path traversal and injections
- **Robust error handling**: Structured JSON responses with appropriate HTTP codes
- **Streaming threshold**: Configurable to optimize memory vs performance

## Architecture

The system is organized into several layers:

### AppState
Shared state across all handlers containing references to cache manager, base URL configuration, and streaming parameters.

### Handlers
Asynchronous functions handling HTTP routes. Each handler is responsible for a specific functionality.

### File Serving Pipeline
4-step process to serve files: parsing, resolution, RAM cache, disk fallback.

## Available Routes

### GET /servers
Lists all available servers with their metadata.

### GET /{server}.json
Returns the complete VersionBuilder JSON for a server.

### GET /{server}/{path}
Serves a specific file (JAR, mod, asset, etc.) with intelligent caching.

### POST /rescan/{server}
Triggers manual rescan of a specific server.

## Integration

This crate integrates with:
- `lighty_cache`: To access version and file cache
- `lighty_models`: For VersionBuilder structures
- `lighty_filesystem`: For disk operations
- `axum`: HTTP framework
