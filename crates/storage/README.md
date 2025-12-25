# Storage Crate

Multi-backend storage abstraction system for Minecraft file distribution with local and S3-compatible support.

## Table of Contents

- [Architecture](docs/architecture.md) - Storage system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and operation flows
- [Errors](docs/errors.md) - Error types documentation
- [Local Backend](docs/local.md) - Local storage implementation
- [S3 Backend](docs/s3.md) - S3-compatible integration (Cloudflare R2, AWS S3, MinIO)

## Overview

The `storage` crate provides a unified abstraction for file storage management, allowing easy switching between different backends without modifying application code.

- **Trait Abstraction**: Common `StorageBackend` interface for all backends
- **Local Backend**: Disk storage with HTTP URL generation
- **S3-Compatible Backend**: Full support for Cloudflare R2, AWS S3, MinIO, DigitalOcean Spaces
- **Async Operations**: Using Tokio for optimal performance
- **Robust Error Handling**: Detailed error types with thiserror

## Architecture

The system is organized around a clear abstraction:

### StorageBackend Trait
Common interface defining storage operations. All backends implement this trait to guarantee compatibility.

### LocalBackend
Backend for local disk storage. Files remain in place and the backend simply generates URLs pointing to the local HTTP server.

### S3Backend
Backend for S3-compatible cloud storage. Uses AWS Rust SDK to communicate with any S3-compatible service via the standard API.

## Integration

This crate integrates with:
- `lighty_cache`: To synchronize modified files to the cloud
- `lighty_config`: For storage backend configuration
- `aws-sdk-s3`: For S3 operations (feature flag `s3`)
