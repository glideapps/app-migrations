use anyhow::{Context, Result};
use glob::glob;
use std::path::Path;

use crate::Migration;

/// Discover all migrations in the given directory.
/// Migrations must match the pattern NNN-name.ext (e.g., 001-init.sh)
pub fn discover_migrations(dir: &Path) -> Result<Vec<Migration>> {
    let pattern = dir.join("[0-9][0-9][0-9]-*");
    let pattern_str = pattern
        .to_str()
        .context("Invalid path for migration directory")?;

    let mut migrations: Vec<Migration> = glob(pattern_str)
        .context("Failed to read glob pattern")?
        .filter_map(|entry| entry.ok())
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let filename = path.file_name()?.to_str()?;
            let prefix = extract_prefix(filename)?;
            let id = extract_id(filename);
            Some(Migration {
                id,
                prefix,
                file_path: path,
            })
        })
        .collect();

    // Sort by prefix to ensure correct execution order
    migrations.sort_by_key(|m| m.prefix);

    Ok(migrations)
}

/// Extract the numeric prefix from a migration filename.
/// Returns None if the filename doesn't start with 3 digits.
pub fn extract_prefix(filename: &str) -> Option<u32> {
    if filename.len() < 3 {
        return None;
    }
    filename[..3].parse().ok()
}

/// Extract the migration ID from a filename.
/// The ID is the filename without extension (e.g., "001-init" from "001-init.sh")
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
    fn test_extract_prefix() {
        assert_eq!(extract_prefix("001-init.sh"), Some(1));
        assert_eq!(extract_prefix("123-something.ts"), Some(123));
        assert_eq!(extract_prefix("999-last.py"), Some(999));
        assert_eq!(extract_prefix("ab-invalid.sh"), None);
        assert_eq!(extract_prefix("1-short.sh"), None);
    }

    #[test]
    fn test_extract_id() {
        assert_eq!(extract_id("001-init.sh"), "001-init");
        assert_eq!(extract_id("002-add-config.ts"), "002-add-config");
        assert_eq!(extract_id("003-no-extension"), "003-no-extension");
    }
}
