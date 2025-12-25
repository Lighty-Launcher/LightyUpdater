# Models Crate

Data structures for representing Minecraft server versions with JSON serialization.

## Table of Contents

- [Architecture](docs/architecture.md) - Data model structure
- [VersionBuilder](docs/version-builder.md) - Version construction and management
- [URL Mapping](docs/url-mapping.md) - O(1) resolution system

## Overview

The `models` crate defines data structures representing a complete Minecraft server version compatible with launchers.

- **VersionBuilder**: Main structure containing all components
- **JSON Serialization**: Compatible with standard launcher format
- **URL Mapping**: HashMap for O(1) URL to path resolution
- **Strongly typed**: Main class, Java version, arguments, etc.
- **Organized collections**: Libraries, mods, natives, assets

## Main Structures

### VersionBuilder
Main container for all server version components.

### MainClass
Java main class for game launch.

### JavaVersion
Required Java version to run the server.

### Arguments
Game and JVM arguments for the launcher.

### Library
Java dependency (JAR) with URL, path, hash, and size.

### Mod
Game modification with metadata similar to libraries.

### Native
Platform-specific native library (Windows/Linux/macOS).

### Client
Minecraft client JAR with hash and size.

### Asset
Game resource (texture, sound, etc.) identified by hash.

## Integration

This crate integrates with:
- `lighty_scanner`: To build VersionBuilders during scan
- `lighty_cache`: To store versions in memory
- `lighty_api`: To serialize to JSON and serve to clients
- `serde`: For JSON serialization/deserialization
