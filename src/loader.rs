use anyhow::{Context, Result};
use glob::glob;
use std::path::Path;

use crate::version::is_valid_version;
use crate::Migration;

/// Discover all migrations in the given directory.
/// Migrations must match the pattern XXXXX-name.ext where XXXXX is a 5-char base36 version
pub fn discover_migrations(dir: &Path) -> Result<Vec<Migration>> {
    // Match 5 alphanumeric characters followed by dash
    let pattern = dir.join("[0-9a-z][0-9a-z][0-9a-z][0-9a-z][0-9a-z]-*");
    let pattern_str = pattern
        .to_str()
        .context("Invalid path for migration directory")?;

    let mut migrations: Vec<Migration> = glob(pattern_str)
        .context("Failed to read glob pattern")?
        .filter_map(|entry| entry.ok())
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let filename = path.file_name()?.to_str()?;
            let version = extract_version(filename)?;
            let id = extract_id(filename);
            Some(Migration {
                id,
                version,
                file_path: path,
            })
        })
        .collect();

    // Sort by version string (lexicographic sort works for base36)
    migrations.sort_by(|a, b| a.version.cmp(&b.version));

    Ok(migrations)
}

/// Extract the version from a migration filename.
/// Returns None if the filename doesn't start with a valid 5-char version.
pub fn extract_version(filename: &str) -> Option<String> {
    if filename.len() < 6 {
        return None;
    }
    // Must have dash after 5-char version
    if filename.as_bytes().get(5) != Some(&b'-') {
        return None;
    }
    let version = &filename[..5];
    if is_valid_version(version) {
        Some(version.to_string())
    } else {
        None
    }
}

/// Extract the migration ID from a filename.
/// The ID is the filename without extension (e.g., "1f72f-init" from "1f72f-init.sh")
pub fn extract_id(filename: &str) -> String {
    // Remove extension if present
    match filename.rfind('.') {
        Some(pos) => filename[..pos].to_string(),
        None => filename.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        assert_eq!(extract_version("1f72f-init.sh"), Some("1f72f".to_string()));
        assert_eq!(
            extract_version("00000-something.ts"),
            Some("00000".to_string())
        );
        assert_eq!(extract_version("zzzzz-last.py"), Some("zzzzz".to_string()));
        assert_eq!(extract_version("ab-invalid.sh"), None); // Too short
        assert_eq!(extract_version("1234-short.sh"), None); // 4 chars, not 5
        assert_eq!(extract_version("123456-toolong.sh"), None); // No dash at position 5
    }

    #[test]
    fn test_extract_id() {
        assert_eq!(extract_id("1f72f-init.sh"), "1f72f-init");
        assert_eq!(extract_id("00000-add-config.ts"), "00000-add-config");
        assert_eq!(extract_id("zzzzz-no-extension"), "zzzzz-no-extension");
    }
}
