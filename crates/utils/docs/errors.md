# Error Documentation

## Main Type: UtilsError

```rust
pub enum UtilsError {
    IoError(#[from] std::io::Error),
    HashError(String),
    PathError(String),
    PathConversionError(String),
}
```

## Error Types

### IoError
I/O error during file reading for hash.
**Causes**: Non-existent file, permissions, disk error.

### HashError
Hash calculation failure.
**Causes**: Corrupted file, read interruption.

### PathError
Invalid or malformed path.
**Causes**: Invalid characters, path too long.

### PathConversionError
Path to string conversion failure.
**Causes**: Non-UTF8 characters, invalid path.
