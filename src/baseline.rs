use anyhow::{bail, Context, Result};
use std::fs;

use crate::state::Baseline;
use crate::{AppliedMigration, Migration};

/// Compare two version strings. Returns true if v1 <= v2.
pub fn version_lte(v1: &str, v2: &str) -> bool {
    v1 <= v2
}

/// Represents an item deleted during baseline cleanup
#[derive(Debug, Clone)]
pub struct DeletedItem {
    pub path: String,
    pub is_directory: bool,
}

/// Delete migration files and associated asset directories at or before the baseline version.
/// Asset directories are identified by having the same prefix as the migration ID (e.g., "1f700-init/").
/// Returns the list of deleted items (files and directories).
pub fn delete_baselined_migrations(
    baseline_version: &str,
    available: &[Migration],
) -> Result<Vec<DeletedItem>> {
    let mut deleted = Vec::new();

    for migration in available {
        if version_lte(&migration.version, baseline_version) {
            // Delete the migration file
            if migration.file_path.exists() {
                fs::remove_file(&migration.file_path).with_context(|| {
                    format!(
                        "Failed to delete migration file: {}",
                        migration.file_path.display()
                    )
                })?;
                deleted.push(DeletedItem {
                    path: migration.file_path.display().to_string(),
                    is_directory: false,
                });
            }

            // Delete associated asset directory if it exists
            // The directory shares the migration ID as its name (e.g., "1f700-init/")
            if let Some(parent) = migration.file_path.parent() {
                let asset_dir = parent.join(&migration.id);
                if asset_dir.exists() && asset_dir.is_dir() {
                    fs::remove_dir_all(&asset_dir).with_context(|| {
                        format!(
                            "Failed to delete migration asset directory: {}",
                            asset_dir.display()
                        )
                    })?;
                    deleted.push(DeletedItem {
                        path: asset_dir.display().to_string(),
                        is_directory: true,
                    });
                }
            }
        }
    }

    Ok(deleted)
}

/// Validate that a baseline can be created at the given version.
/// Returns an error if validation fails.
pub fn validate_baseline(
    version: &str,
    available: &[Migration],
    applied: &[AppliedMigration],
    existing_baseline: Option<&Baseline>,
) -> Result<()> {
    // Check if the version matches any migration
    let matching_migration = available.iter().find(|m| m.version == version);
    if matching_migration.is_none() {
        bail!("No migration found with version '{}'", version);
    }

    // Cannot move baseline backward
    if let Some(existing) = existing_baseline {
        if version < existing.version.as_str() {
            bail!(
                "Cannot move baseline backward from '{}' to '{}'",
                existing.version,
                version
            );
        }
    }

    // All migrations at or before the version must be in history
    let applied_ids: std::collections::HashSet<&str> =
        applied.iter().map(|a| a.id.as_str()).collect();

    for migration in available {
        if version_lte(&migration.version, version) && !applied_ids.contains(migration.id.as_str())
        {
            bail!(
                "Cannot baseline: migration '{}' has not been applied",
                migration.id
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_version_lte() {
        assert!(version_lte("1f700", "1f700"));
        assert!(version_lte("1f700", "1f710"));
        assert!(!version_lte("1f710", "1f700"));
        assert!(version_lte("00000", "zzzzz"));
    }

    #[test]
    fn test_validate_baseline_no_matching_migration() {
        let available = vec![Migration {
            id: "1f700-first".to_string(),
            version: "1f700".to_string(),
            file_path: PathBuf::from("1f700-first.sh"),
        }];
        let applied = vec![];

        let result = validate_baseline("1f800", &available, &applied, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No migration found"));
    }

    #[test]
    fn test_validate_baseline_unapplied_migration() {
        let available = vec![
            Migration {
                id: "1f700-first".to_string(),
                version: "1f700".to_string(),
                file_path: PathBuf::from("1f700-first.sh"),
            },
            Migration {
                id: "1f710-second".to_string(),
                version: "1f710".to_string(),
                file_path: PathBuf::from("1f710-second.sh"),
            },
        ];
        let applied = vec![AppliedMigration {
            id: "1f710-second".to_string(),
            applied_at: Utc::now(),
        }];

        // Try to baseline at 1f710, but 1f700 hasn't been applied
        let result = validate_baseline("1f710", &available, &applied, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("has not been applied"));
    }

    #[test]
    fn test_validate_baseline_backward_movement() {
        let available = vec![
            Migration {
                id: "1f700-first".to_string(),
                version: "1f700".to_string(),
                file_path: PathBuf::from("1f700-first.sh"),
            },
            Migration {
                id: "1f710-second".to_string(),
                version: "1f710".to_string(),
                file_path: PathBuf::from("1f710-second.sh"),
            },
        ];
        let applied = vec![
            AppliedMigration {
                id: "1f700-first".to_string(),
                applied_at: Utc::now(),
            },
            AppliedMigration {
                id: "1f710-second".to_string(),
                applied_at: Utc::now(),
            },
        ];

        let existing = Baseline {
            version: "1f710".to_string(),
            created: Utc::now(),
            summary: None,
        };

        let result = validate_baseline("1f700", &available, &applied, Some(&existing));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("backward"));
    }

    #[test]
    fn test_validate_baseline_success() {
        let available = vec![
            Migration {
                id: "1f700-first".to_string(),
                version: "1f700".to_string(),
                file_path: PathBuf::from("1f700-first.sh"),
            },
            Migration {
                id: "1f710-second".to_string(),
                version: "1f710".to_string(),
                file_path: PathBuf::from("1f710-second.sh"),
            },
        ];
        let applied = vec![
            AppliedMigration {
                id: "1f700-first".to_string(),
                applied_at: Utc::now(),
            },
            AppliedMigration {
                id: "1f710-second".to_string(),
                applied_at: Utc::now(),
            },
        ];

        let result = validate_baseline("1f710", &available, &applied, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_baselined_migrations_with_asset_dirs() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let migrations_dir = temp_dir.path();

        // Create migration file
        let migration_file = migrations_dir.join("1f700-first.sh");
        fs::write(&migration_file, "#!/bin/bash\necho hello").unwrap();

        // Create asset directory with files
        let asset_dir = migrations_dir.join("1f700-first");
        fs::create_dir(&asset_dir).unwrap();
        fs::write(asset_dir.join("config.json"), "{}").unwrap();
        fs::write(asset_dir.join("template.txt"), "template").unwrap();

        // Create a second migration without asset dir
        let migration_file2 = migrations_dir.join("1f710-second.sh");
        fs::write(&migration_file2, "#!/bin/bash\necho world").unwrap();

        let available = vec![
            Migration {
                id: "1f700-first".to_string(),
                version: "1f700".to_string(),
                file_path: migration_file.clone(),
            },
            Migration {
                id: "1f710-second".to_string(),
                version: "1f710".to_string(),
                file_path: migration_file2.clone(),
            },
        ];

        // Delete migrations at or before 1f710
        let deleted = delete_baselined_migrations("1f710", &available).unwrap();

        // Should delete both files and the asset directory
        assert_eq!(deleted.len(), 3); // 2 files + 1 directory

        let files: Vec<_> = deleted.iter().filter(|d| !d.is_directory).collect();
        let dirs: Vec<_> = deleted.iter().filter(|d| d.is_directory).collect();

        assert_eq!(files.len(), 2);
        assert_eq!(dirs.len(), 1);

        // Verify files are gone
        assert!(!migration_file.exists());
        assert!(!migration_file2.exists());
        assert!(!asset_dir.exists());
    }

    #[test]
    fn test_delete_baselined_migrations_no_asset_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let migrations_dir = temp_dir.path();

        // Create migration file without asset directory
        let migration_file = migrations_dir.join("1f700-first.sh");
        fs::write(&migration_file, "#!/bin/bash\necho hello").unwrap();

        let available = vec![Migration {
            id: "1f700-first".to_string(),
            version: "1f700".to_string(),
            file_path: migration_file.clone(),
        }];

        let deleted = delete_baselined_migrations("1f700", &available).unwrap();

        // Should only delete the file
        assert_eq!(deleted.len(), 1);
        assert!(!deleted[0].is_directory);
        assert!(!migration_file.exists());
    }
}
