# Utils Crate

Utilities for checksum calculation and path normalization.

## Table of Contents

- [Checksums](docs/checksums.md) - Asynchronous and synchronous SHA1 calculation
- [Path utilities](docs/path.md) - Normalization and Maven conversion
- [Errors](docs/errors.md) - Error documentation

## Overview

The `utils` crate provides essential utilities for file and path operations.

- **SHA1 calculation**: Asynchronous and synchronous with streaming
- **Path normalization**: Windows/Unix conversion
- **Maven naming**: Path to Maven notation conversion
- **Configurable buffer**: Memory vs performance optimization

## Main Functions

### compute_sha1
Calculates the SHA1 hash of a file asynchronously.

### compute_sha1_with_size
Calculates SHA1 and also returns file size.

### compute_sha1_sync
Synchronous version of hash calculation.

### path_to_maven_name
Converts a library path to Maven notation (group:artifact:version).

### normalize_path
Normalizes path separators for cross-platform compatibility.

## Integration

This crate integrates with:
- `lighty_scanner`: To calculate hashes of scanned files
- `lighty_cache`: To verify file integrity
- `sha1`: For hashing algorithm
- `tokio`: For asynchronous I/O
