# Overview

`lighty-config` is the **schema** crate: every `struct`, `enum` and
default that describes a valid `config.toml`. It has zero I/O —
`serde::Deserialize` does the parsing, but the actual file reading,
migration and validation live in `lighty-adapters`.

Splitting schema and I/O keeps this crate dependency-free (just
`serde`) and lets the runtime, the API, the cache and the rescan
each take a typed `Arc<RwLock<Config>>` without pulling TOML or
filesystem code into their dep graph.

## What's inside

| Item | Purpose |
|---|---|
| `Config` | Root struct (`server`, `cache`, `storage`, `cdn`, `cloudflare`, `hot_reload`, `servers`). |
| `ServerSettings`, `CacheSettings`, `StorageSettings`, `CdnSettings`, `CloudflareSettings`, `HotReloadSettings`, `BatchConfig` | Sub-sections. |
| `ServerConfig` | One per server in `[[servers]]`: name, base path, enable flags, loader, mc_version, java_version, main_class, args, etc. |
| `StorageBackend` | Enum `{ Local, S3 }` selected by `config.storage.backend`. |
| `ConfigError` | One error type; only used for in-process validation messages (the watcher / loader use it). |

## Section layout

```mermaid
flowchart LR
    C[Config] --> S1[server]
    C --> S2[cache]
    C --> S3[storage]
    C --> S4[cdn]
    C --> S5[cloudflare]
    C --> S6[hot_reload]
    C --> Servers["[[servers]]"]

    Servers --> SC1[ServerConfig 1]
    Servers --> SC2[ServerConfig N]

    S3 --> Backend{backend: Local | S3}
```

The actual file layout follows TOML conventions: each subsection
maps to `[server]`, `[cache]`, etc., and each `[[servers]]` entry is
a TOML array-of-tables.

## Design notes

- **`Arc<str>` everywhere strings are stable.** Server name, base
  path, base URL etc. are `Arc<str>` rather than `String` — they
  rarely change at runtime and are cheap to share across the cache,
  the rescan loop, and the HTTP handlers.
- **No defaults inside `#[derive(Deserialize)]` annotations.** The
  default values live in `defaults.rs` and are applied during
  migration (`lighty-adapters::config_io_migration`). This keeps
  schema and defaults decoupled.

## Cargo features

None.

## See also

- [`how-to-use.md`](./how-to-use.md)
- [`exports.md`](./exports.md)
- [`architecture.md`](./architecture.md), [`migration.md`](./migration.md),
  [`hot-reload.md`](./hot-reload.md)
- [`../../adapters/docs/overview.md`](../../adapters/docs/overview.md) —
  loader + migration code
- [`../../watcher/docs/overview.md`](../../watcher/docs/overview.md) —
  hot-reload pipeline
