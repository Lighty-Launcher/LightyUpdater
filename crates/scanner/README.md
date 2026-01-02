# Scanner Crate

Parallelized scanning system for analyzing and indexing Minecraft server structures.

## Table of Contents

- [Architecture](docs/architecture.md) - Scan system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and processes
- [Errors](docs/errors.md) - Error types documentation
- [JarScanner](docs/jar-scanner.md) - Parallelized JAR file scanning
- [Specialized Scanners](docs/specialized-scanners.md) - Client, Libraries, Mods, Natives, Assets

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
