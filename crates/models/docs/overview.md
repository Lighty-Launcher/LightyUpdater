# Overview

`lighty-models` holds the domain types every other crate operates on.
Pure data, no logic beyond `build_url_map` /
`add_url_mapping` / `remove_url_mapping` on `VersionBuilder`. The
crate has only `serde` as a dep.

## What's inside

| Type | Purpose |
|---|---|
| `VersionBuilder` | The full snapshot of a server: `main_class`, `java_version`, `arguments`, `libraries`, `mods`, `natives`, `client`, `assets`, plus the in-memory `url_to_path_map`. |
| `MainClass`, `JavaVersion`, `Arguments` | Sub-records for the launch contract. |
| `Library` | Maven-style coords + URL + path + sha1 + size. |
| `Mod` | Same shape as `Library` for jar mods (Fabric/Forge/NeoForge/Quilt). |
| `Native` | OS-specific natives with explicit `os` tag. |
| `Client` | The single Minecraft client jar. |
| `Asset` | Game asset (`hash`, `size`, optional `url`/`path`). |

## URL map

The interesting piece: `VersionBuilder::url_to_path_map` is rebuilt
from the per-entry `url` and `path` so HTTP handlers can resolve a
public CDN URL back to an on-disk relative path in O(1).

```mermaid
flowchart LR
    Client[client] --> Map
    Libraries[libraries[i]] --> Map
    Mods[mods[i]] --> Map
    Natives[natives[i]] --> Map
    Assets[assets[i]] --> Map

    Map[url_to_path_map<br/>HashMap&lt;String,String&gt;]
```

Two ways to maintain it:

- `build_url_map(&mut self)` — full rebuild from scratch.
- `add_url_mapping` / `remove_url_mapping` — incremental, called by
  `lighty-file-diff::FileDiff::apply_to_url_map`.

The map is `#[serde(skip)]` so it never lands in the on-the-wire
JSON.

## Cargo features

None.

## See also

- [`how-to-use.md`](./how-to-use.md) — construct + serialize examples
- [`exports.md`](./exports.md)
- [`version-builder.md`](./version-builder.md), [`url-mapping.md`](./url-mapping.md)
- [`../../file-diff/docs/overview.md`](../../file-diff/docs/overview.md) —
  the diff that lives on top of these types
