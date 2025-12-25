# Available Operations

## ensure_server_structure

Creates the complete folder structure for a server.

**Signature**:
```rust
async fn ensure_server_structure(
    base_path: &str,
    server_folder: &str
) -> Result<PathBuf>
```

**Example**:
```rust
let path = FileSystem::ensure_server_structure(
    "/var/minecraft",
    "server1"
).await?;
// Returns: /var/minecraft/server1
```

**Created folders**:
- Root directory
- client/
- libraries/
- mods/
- natives/ + subdirectories
- assets/

**Possible errors**:
- Insufficient permissions
- Disk full
- Invalid path

---

## build_server_path

Builds the complete path for a server without creation.

**Signature**:
```rust
fn build_server_path(
    base_path: &str,
    server_folder: &str
) -> PathBuf
```

**Example**:
```rust
let path = FileSystem::build_server_path(
    "/var/minecraft",
    "server1"
);
// Returns: /var/minecraft/server1
```

**Usage**: Path construction for read access.

---

## get_absolute_path_string

Converts a path (absolute or relative) to an absolute string.

**Signature**:
```rust
fn get_absolute_path_string(path: &str) -> Result<String>
```

**Example**:
```rust
// From directory /home/user
let abs = FileSystem::get_absolute_path_string("servers")?;
// Returns: "/home/user/servers"

let abs = FileSystem::get_absolute_path_string("/var/minecraft")?;
// Returns: "/var/minecraft"
```

**Usage**: Configuration path normalization.

---

## create_directory (private)

Creates a folder with logging.

**Behavior**:
- If exists: Log debug "Exists"
- If doesn't exist: Create + log debug "Created"
- Recursive via create_dir_all

---

## get_absolute_path (private)

Resolution from relative to absolute path.

**Algorithm**:
1. Check if path is already absolute
2. If not: Join with current_dir()
3. Return absolute PathBuf
