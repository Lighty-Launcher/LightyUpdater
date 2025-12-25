# Filesystem Crate

File system operations for managing Minecraft server structure.

## Table of Contents

- [Architecture](docs/architecture.md) - Filesystem operations architecture
- [Server Structure](docs/server-structure.md) - Folder organization
- [Operations](docs/operations.md) - Available operations

## Overview

The `filesystem` crate provides utilities to create and manage Minecraft server folder structure.

- **Structure creation**: Standardized folders for each server
- **Asynchronous operations**: Uses Tokio for non-blocking I/O
- **Absolute paths**: Automatic resolution of relative paths
- **Idempotence**: Safe operations if folders already exist

## Standard Structure

Each server has the following structure:
```
server_name/
├── client/          # Minecraft client JAR
├── libraries/       # Java libraries
├── mods/            # Game modifications
├── natives/         # Native libraries
│   ├── windows/     # Windows natives
│   ├── linux/       # Linux natives
│   └── macos/       # macOS natives
└── assets/          # Game resources
```

## Main Operations

### ensure_server_structure
Creates the complete folder structure for a server.

### build_server_path
Builds the complete path for a server.

### get_absolute_path_string
Converts a relative path to an absolute path.

## Integration

This crate integrates with:
- `lighty_watcher`: To create folders for new servers
- `lighty_scanner`: To locate files to scan
- `lighty_api`: To build paths during serving
- `tokio`: For asynchronous I/O
- `anyhow`: For error handling
