# Using lighty-models

## 1. Build a minimal `VersionBuilder` (tests, fixtures)

```rust
use lighty_models::{Arguments, JavaVersion, MainClass, VersionBuilder};

let builder = VersionBuilder {
    main_class: MainClass { main_class: "net.minecraft.client.main.Main".into() },
    java_version: JavaVersion { major_version: 21 },
    arguments: Arguments { game: vec![], jvm: vec![] },
    libraries: vec![],
    mods: vec![],
    natives: None,
    client: None,
    assets: vec![],
    url_to_path_map: Default::default(),
};
```

## 2. Serialize a snapshot to JSON

```rust
let json = serde_json::to_string(&builder)?;
```

`url_to_path_map` is `#[serde(skip)]` — it stays out of the on-the-wire
payload. The HTTP handler rebuilds it on deserialization via
`build_url_map`.

## 3. Maintain the URL map

After deserializing a `VersionBuilder` from somewhere (e.g. a saved
cache snapshot), call `build_url_map` once:

```rust
builder.build_url_map();
```

For incremental updates (after a `FileDiff`), prefer:

```rust
diff.apply_to_url_map(&mut builder);
```

(That helper lives in `lighty-file-diff`.)

## 4. Resolve a URL → path

```rust
let path = builder.url_to_path_map.get(&request_url).cloned();
```

Returns the relative on-disk path under the server's working dir,
e.g. `mods/iris-1.8.0.jar`. The HTTP handler then prepends the
server name and the base path.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`version-builder.md`](./version-builder.md)
- [`url-mapping.md`](./url-mapping.md)
