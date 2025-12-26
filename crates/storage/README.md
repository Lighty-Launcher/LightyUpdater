# Storage Crate

Multi-backend storage abstraction system for Minecraft file distribution with local and S3-compatible support.

## Table of Contents

- [Architecture](docs/architecture.md) - Storage system architecture
- [Processing Flow](docs/flow.md) - Sequence diagrams and operation flows
- [Errors](docs/errors.md) - Error types documentation
- [Local Backend](docs/local.md) - Local storage implementation
- [S3 Backend](docs/s3.md) - S3-compatible integration (Cloudflare R2, AWS S3, MinIO)

## Integration

This crate integrates with:
- `lighty_cache`: To synchronize modified files to the cloud
- `lighty_config`: For storage backend configuration
- `aws-sdk-s3`: For S3 operations (feature flag `s3`)
