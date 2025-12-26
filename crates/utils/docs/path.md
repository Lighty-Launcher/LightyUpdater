# Path Utilities

## path_to_maven_name

Converts a library path to Maven notation.

**Maven Format**: `group:artifact:version`

**Algorithm**:
```mermaid
flowchart TD
    Path[Path components<br/>com,google,guava,31.0,guava-31.0.jar] --> Extract[Extract components]

    Extract --> Group[group = components from 0 to -3<br/>com.google.guava]
    Extract --> Artifact[artifact = components at -3<br/>guava]
    Extract --> Version[version = components at -2<br/>31.0]

    Group --> Format[Format: group:artifact:version]
    Artifact --> Format
    Version --> Format

    Format --> Result[com.google.guava:guava:31.0]
```

**Example**:
```rust
let path = Path::new("com/google/guava/31.0/guava-31.0.jar");
let maven = path_to_maven_name(path);
// "com.google.guava:guava:31.0"
```

## normalize_path

Normalizes separators for cross-platform compatibility.

**Transformation**: `\` to `/`

**Examples**:
```rust
// Windows path
normalize_path(Path::new("mods\\JEI-1.20.1.jar"))
// "mods/JEI-1.20.1.jar"

// Unix path (unchanged)
normalize_path(Path::new("mods/JEI-1.20.1.jar"))
// "mods/JEI-1.20.1.jar"
```

**Rationale**: URLs always use `/`, normalization necessary on Windows.
