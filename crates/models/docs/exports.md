# Exports

Public surface of `lighty-models`.

## Crate root

```rust
use lighty_models::{
    Arguments,
    Asset,
    Client,
    JavaVersion,
    Library,
    MainClass,
    Mod,
    Native,
    VersionBuilder,
};
```

## `VersionBuilder`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBuilder {
    pub main_class:   MainClass,
    pub java_version: JavaVersion,
    pub arguments:    Arguments,
    pub libraries:    Vec<Library>,
    pub mods:         Vec<Mod>,
    pub natives:      Option<Vec<Native>>,
    pub client:       Option<Client>,
    pub assets:       Vec<Asset>,
    #[serde(skip)]
    pub url_to_path_map: HashMap<String, String>,
}

impl VersionBuilder {
    pub fn build_url_map(&mut self);
    pub fn add_url_mapping(&mut self, url: String, path: String);
    pub fn remove_url_mapping(&mut self, url: &str);
}
```

## Sub-records

```rust
pub struct MainClass    { pub main_class: String }
pub struct JavaVersion  { pub major_version: u8 }
pub struct Arguments    { pub game: Vec<String>, pub jvm: Vec<String> }

pub struct Library {
    pub name: String,
    pub url:  Option<String>,
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

pub struct Mod {
    pub name: String,
    pub url:  Option<String>,
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

pub struct Native {
    pub name: String,
    pub url:  String,
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub os:   String,   // "windows" | "linux" | "macos"
}

pub struct Client {
    pub name: String,
    pub url:  String,
    pub path: String,
    pub sha1: String,
    pub size: u64,
}

pub struct Asset {
    pub hash: String,
    pub size: u64,
    pub url:  Option<String>,
    pub path: Option<String>,
}
```

All sub-records derive `Debug + Clone + Serialize + Deserialize`.

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`version-builder.md`](./version-builder.md)
- [`url-mapping.md`](./url-mapping.md)
