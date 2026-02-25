# Config Crate

Pure configuration model crate for LightyUpdater.

## Responsibilities

- Define all serializable config structures.
- Provide serde defaults for optional fields.
- Expose the default config template string.

## Non-Responsibilities

- No file I/O.
- No migration logic.
- No event emission.

## Related Crate

- `lighty-config-io` handles loading, migration, and config-file lifecycle.
