use crate::errors::ApiError;

/// Validates a path component to prevent path traversal attacks
pub fn validate_path_component(component: &str) -> Result<(), ApiError> {
    // Check for path traversal attempts
    if component.contains("..") {
        return Err(ApiError::InvalidPath(
            "Path contains '..' (path traversal attempt)".to_string()
        ));
    }

    // Check for null bytes
    if component.contains('\0') {
        return Err(ApiError::InvalidPath(
            "Path contains null byte".to_string()
        ));
    }

    // Check for absolute paths
    if component.starts_with('/') || component.starts_with('\\') {
        return Err(ApiError::InvalidPath(
            "Absolute paths are not allowed".to_string()
        ));
    }

    // Check for Windows drive letters (C:, D:, etc.)
    if component.len() >= 2 && component.chars().nth(1) == Some(':') {
        return Err(ApiError::InvalidPath(
            "Drive letters are not allowed".to_string()
        ));
    }

    Ok(())
}
