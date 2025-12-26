# File System Architecture

## FileSystem Structure

**FileSystem** is an empty struct serving as a namespace for utility functions.

**Public Methods**:
- `ensure_server_structure(base_path, server_folder)`: Create complete server directory structure
- `build_server_path(base_path, server_folder)`: Build server path
- `get_absolute_path_string(path)`: Get absolute path as String

**Private Methods**:
- `create_directory(path, description)`: Create single directory
- `get_absolute_path(path)`: Convert to absolute PathBuf

## Structure Creation Flow

```mermaid
graph TD
    Start[ensure_server_structure] --> BuildPath[Build full path]
    BuildPath --> Absolute[Convert to absolute]

    Absolute --> Root[Create root directory]
    Root --> Client[Create client/]
    Client --> Libs[Create libraries/]
    Libs --> Mods[Create mods/]
    Mods --> NativesRoot[Create natives/]

    NativesRoot --> Win[Create natives/windows/]
    NativesRoot --> Linux[Create natives/linux/]
    NativesRoot --> Mac[Create natives/macos/]

    Win --> Assets[Create assets/]
    Linux --> Assets
    Mac --> Assets

    Assets --> Return[Return absolute path]
```

## Atomic Operations

Each create_directory operation is atomic and idempotent:

```mermaid
sequenceDiagram
    participant FS as FileSystem
    participant Tokio
    participant Disk

    FS->>Disk: Check if path exists
    alt Path exists
        Disk-->>FS: true
        FS->>FS: Log "Exists"
    else Path not exists
        Disk-->>FS: false
        FS->>Tokio: create_dir_all
        Tokio->>Disk: Create directory
        Disk-->>FS: Ok
        FS->>FS: Log "Created"
    end
```

## Path Resolution

```mermaid
graph TD
    Input[Input path] --> Check{Is absolute?}

    Check -->|Yes| Return1[Use as-is]
    Check -->|No| CWD[Get current working directory]

    CWD --> Join[Join CWD + path]
    Join --> Return2[Return absolute path]
```
