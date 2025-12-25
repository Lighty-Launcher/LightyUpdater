# Scanner Crate

Parallelized scanning system for analyzing and indexing Minecraft server structures.

## Table of Contents

- [Architecture](docs/architecture.md) - Scan system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and processes
- [Errors](docs/errors.md) - Error types documentation
- [JarScanner](docs/jar-scanner.md) - Parallelized JAR file scanning
- [Specialized Scanners](docs/specialized-scanners.md) - Client, Libraries, Mods, Natives, Assets

## Overview

The `scanner` crate provides a high-performance scanning system for analyzing Minecraft server structures and generating complete metadata. It is optimized for:

- **Massive parallelization**: Uses tokio and semaphores to scan multiple files simultaneously
- **Concurrency control**: Configurable batch processing to avoid system overload
- **Asynchronous hash calculation**: SHA1 computed asynchronously with configurable buffer
- **Multi-type support**: Client JAR, libraries, mods, natives (multi-OS), assets

## Architecture

The system is organized around several components:

### ServerScanner
Main entry point that coordinates the complete scan of a server. It builds a `VersionBuilder` containing all metadata necessary for the launcher.

### JarScanner
Generic and reusable scanner for JAR files with:
- Controlled parallelization via semaphore
- Customizable result mapping
- Automatic error handling
- Support for nested structures

### Specialized Scanners
Each file type has its dedicated scanner:
- **Client**: Searches for the client JAR in the `client/` folder
- **Libraries**: Recursive scan with conversion to Maven notation
- **Mods**: Simple scan of JAR files in `mods/`
- **Natives**: Multi-OS scan (windows, linux, macos) with platform organization
- **Assets**: Recursive scan of all files in `assets/`

## Performance

The system is designed for optimal performance:

### Parallelization
- Scan multiple files simultaneously with `futures::stream`
- Concurrency control via `tokio::sync::Semaphore`
- Configurable buffer for hash calculation

### Optimizations
- Asynchronous hash calculation to avoid blocking the runtime
- Collect paths first, then parallel processing
- Optimized filter and map operations with iterators
- `buffer_unordered` to maximize concurrency

### Configuration
Configurable batch sizes per file type:
- Client: Usually 1 file
- Libraries: 50-100 files in parallel
- Mods: 50-100 files in parallel
- Natives: 50-100 files in parallel
- Assets: 50-100 files in parallel (can be thousands)

## Integration

This crate integrates with:
- `lighty_models`: For data structures (VersionBuilder, Library, Mod, etc.)
- `lighty_storage`: To generate file URLs
- `lighty_utils`: For utilities (SHA1 calculation, path normalization, Maven conversion)
- `lighty_config`: For batch size configuration

## Expected Server Structure

```
server_name/
├── client/
│   └── client.jar
├── libraries/
│   ├── com/
│   │   └── example/
│   │       └── library/
│   │           └── 1.0.0/
│   │               └── library-1.0.0.jar
│   └── ...
├── mods/
│   ├── optifine.jar
│   ├── jei.jar
│   └── ...
├── natives/
│   ├── windows/
│   │   └── lwjgl-natives-windows.jar
│   ├── linux/
│   │   └── lwjgl-natives-linux.jar
│   └── macos/
│       └── lwjgl-natives-macos.jar
└── assets/
    ├── minecraft/
    │   ├── textures/
    │   ├── sounds/
    │   └── ...
    └── ...
```

## Scan Results

The scan produces a `VersionBuilder` containing:
- Server metadata (main class, Java version, arguments)
- Client JAR with SHA1, size, URL
- List of libraries with Maven notation
- List of mods
- Natives organized by OS
- Assets with relative paths
- URL map for fast file→URL resolution
