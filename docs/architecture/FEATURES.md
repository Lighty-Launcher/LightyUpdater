# Feature Flags Matrix

Snapshot date: 2026-05-24

## Declared features

| Crate           | Feature  | Enables                                             |
|-----------------|----------|-----------------------------------------------------|
| `lighty-storage`| `s3`     | `aws-sdk-s3`, `aws-config`, `aws-credential-types`  |
| `lighty-runtime`| `s3`     | `lighty-storage/s3`                                 |
| `lighty-app`    | `s3`     | `lighty-runtime/s3`                                 |
| `lighty-cli`    | `s3`     | `lighty-runtime/s3`                                 |

There is currently a single user-facing flag (`s3`) that propagates from the binaries down to `lighty-storage`. CDN and Cloudflare integrations are runtime-toggleable via config, not compile-time features.

## Supported build combinations

| Command                                                      | Backend(s)                  | Status |
|--------------------------------------------------------------|-----------------------------|--------|
| `cargo build --workspace`                                    | Local filesystem only       | OK     |
| `cargo build --workspace --no-default-features`              | Local filesystem only       | OK     |
| `cargo build --workspace --features s3`                      | Local + S3 / Cloudflare R2  | OK     |
| `cargo build -p lighty-app --features s3`                    | Same as above, app binary   | OK     |
| `cargo build -p lighty --features s3`                        | Same as above, cli binary   | OK     |

All combinations were verified locally on 2026-05-24.

## Runtime toggles (not compile-time)

| Section in `config.toml` | Effect when `enabled = false`                              |
|--------------------------|------------------------------------------------------------|
| `[cdn]`                  | `lighty-runtime` does not construct a `CdnClient`; `CdnEventSink` simply ignores `CdnPurgeRequested` events |
| `[cloudflare]`           | Same as above for `CloudflareClient` / `CloudflarePurgeRequested` |
| `[cache]` `enabled`      | `CacheManager::initialize` and `start_auto_rescan` become no-ops |
| `[cache]` `auto_scan`    | Initial scan is skipped on boot                            |
| `[cache]` `rescan_interval` | `0` switches to file-watcher mode; non-zero is timer-based |

Because CDN and Cloudflare are config-driven (not feature-gated), there is no combinatorial explosion to test: the build matrix is `{default, s3}` and runtime variants are exercised through integration tests in each crate.

## Recommendation

Keep the `s3` feature as the only compile-time toggle. If a new backend (e.g. GCS, Azure Blob) is added, mirror the existing pattern: feature defined on `lighty-storage`, propagated upward through `lighty-runtime`, `lighty-app`, `lighty-cli`.

Avoid introducing feature flags for CDN/Cloudflare. They are HTTP clients without heavy native dependencies, so the cost of always compiling them is negligible compared to the testing burden of an extra feature dimension.
