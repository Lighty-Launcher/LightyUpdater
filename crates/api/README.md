# API Crate

HTTP API layer for Minecraft file distribution with intelligent RAM cache management and optimized streaming.

## Table of Contents

- [Architecture](docs/architecture.md) - HTTP API architecture
- [Processing Flow](docs/flow.md) - Route sequence diagrams
- [Errors](docs/errors.md) - Error types documentation
- [Handlers](docs/handlers.md) - Routes and handlers documentation
- [File Serving](docs/file-serving.md) - File distribution system
- [Resolution](docs/resolution.md) - URL to file path resolution

## Available Routes

### GET /servers
Lists all available servers with their metadata.

### GET /{server}.json
Returns the complete VersionBuilder JSON for a server.

### GET /{server}/{path}
Serves a specific file (JAR, mod, asset, etc.) with intelligent caching.

## Integration

This crate integrates with:
- `lighty_cache`: To access version and file cache
- `lighty_models`: For VersionBuilder structures
- `lighty_filesystem`: For disk operations
- `axum`: HTTP framework
