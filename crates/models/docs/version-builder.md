# VersionBuilder - Version Construction

## URL Mapping Construction

```mermaid
flowchart TD
    Start[New VersionBuilder] --> BuildMap[build_url_map]

    BuildMap --> Client{Client exists?}
    Client -->|Yes| AddClient[Add client URL mapping]
    Client -->|No| Libraries

    AddClient --> Libraries[For each library]
    Libraries --> CheckLib{Has URL & path?}
    CheckLib -->|Yes| AddLib[Add library mapping]
    CheckLib -->|No| NextLib[Next library]
    AddLib --> NextLib
    NextLib --> Libraries

    Libraries --> Mods[For each mod]
    Mods --> AddMod[Add mod mapping]
    AddMod --> Mods

    Mods --> Natives{Natives exist?}
    Natives -->|Yes| AddNatives[Add native mappings]
    Natives -->|No| Assets

    AddNatives --> Assets[For each asset]
    Assets --> AddAsset[Add asset mapping]
    AddAsset --> Assets

    Assets --> Complete[Mapping complete]
```

## Incremental Updates

```rust
// Add a file
version.add_url_mapping(
    "http://localhost/server/mods/new-mod.jar".to_string(),
    "mods/new-mod.jar".to_string()
);

// Remove a file
version.remove_url_mapping("http://localhost/server/mods/old-mod.jar");

// Modification (remove then add)
version.remove_url_mapping(old_url);
version.add_url_mapping(new_url, new_path);
```

## Validation

VersionBuilder does not validate data. Scanner's responsibility:
- Verify file existence
- Calculate correct hashes
- Validate sizes
- Ensure unique URLs
