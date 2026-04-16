//! Path traversal protection utilities

use crate::error::Result;
use std::path::{Component, Path, PathBuf};

/// Safely resolve a path for read operations where the target must exist
pub fn safe_resolve_path(base: &Path, path: &str) -> Result<PathBuf> {
    let p = Path::new(path);

    // Phase 1: Reject any path with ".." components
    for component in p.components() {
        if matches!(component, Component::ParentDir) {
            return Err(crate::error::Error::PermissionDenied(
                "Path traversal denied: '..' components forbidden".into(),
            ));
        }
    }

    // Phase 2: Join with base path
    let joined = base.join(p);

    // Phase 3: Canonicalize to resolve symlinks
    let canonical = joined.canonicalize()?;

    // Phase 4: Final verification that path is still inside base
    if !canonical.starts_with(base) {
        return Err(crate::error::Error::PermissionDenied(format!(
            "Path traversal denied: '{}' is outside base directory '{}'",
            canonical.display(),
            base.display()
        )));
    }

    Ok(canonical)
}

/// Safely resolve a path for write operations where the target may not exist yet
pub fn safe_resolve_parent(base: &Path, path: &str) -> Result<PathBuf> {
    let p = Path::new(path);

    // Phase 1: Reject ".." in any component
    for component in p.components() {
        if matches!(component, Component::ParentDir) {
            return Err(crate::error::Error::PermissionDenied(
                "Path traversal denied: '..' components forbidden".into(),
            ));
        }
    }

    // Phase 2: Split into parent and filename
    let parent = p
        .parent()
        .filter(|par| !par.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(""));

    let filename = p.file_name().ok_or_else(|| {
        crate::error::Error::PermissionDenied("Invalid path: no file name".into())
    })?;

    // Phase 3: Canonicalize the parent directory
    let canonical_parent = if parent == Path::new("") {
        base.canonicalize()?
    } else {
        base.join(parent).canonicalize()?
    };

    // Phase 4: Verify parent is still inside base
    if !canonical_parent.starts_with(base) {
        return Err(crate::error::Error::PermissionDenied(format!(
            "Path traversal denied: parent directory '{}' is outside base directory '{}'",
            canonical_parent.display(),
            base.display()
        )));
    }

    // Phase 5: Belt-and-suspenders check on filename
    let filename_str = filename.to_string_lossy();
    if filename_str.contains("..") {
        return Err(crate::error::Error::PermissionDenied(
            "Path traversal denied in file name".into(),
        ));
    }

    Ok(canonical_parent.join(filename))
}

/// Check if a path is safe and contains no traversal attempts
pub fn is_path_safe(path: &str) -> bool {
    let p = Path::new(path);

    for component in p.components() {
        if matches!(component, Component::ParentDir) {
            return false;
        }
    }

    if let Some(filename) = p.file_name() {
        if filename.to_string_lossy().contains("..") {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_is_path_safe() {
        assert!(is_path_safe("test.txt"));
        assert!(is_path_safe("subdir/test.txt"));
        assert!(!is_path_safe("../test.txt"));
        assert!(!is_path_safe("subdir/../test.txt"));
        assert!(!is_path_safe("test..txt"));
    }

    #[test]
    fn test_safe_resolve_path() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        std::fs::create_dir(base.join("subdir")).unwrap();
        std::fs::write(base.join("test.txt"), "test").unwrap();
        std::fs::write(base.join("subdir/test.txt"), "test").unwrap();

        assert!(safe_resolve_path(base, "test.txt").is_ok());
        assert!(safe_resolve_path(base, "subdir/test.txt").is_ok());
        assert!(safe_resolve_path(base, "../test.txt").is_err());
        assert!(safe_resolve_path(base, "subdir/../test.txt").is_err());
    }
}
