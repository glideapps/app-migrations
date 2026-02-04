use anyhow::{bail, Context, Result};
use std::fs;

use crate::state::Baseline;
use crate::{AppliedMigration, Migration};

/// Compare two version strings. Returns true if v1 <= v2.
pub fn version_lte(v1: &str, v2: &str) -> bool {
    v1 <= v2
}

/// Delete migration files at or before the baseline version.
/// Returns the list of deleted file paths.
pub fn delete_baselined_migrations(
    baseline_version: &str,
    available: &[Migration],
) -> Result<Vec<String>> {
    let mut deleted = Vec::new();

    for migration in available {
        if version_lte(&migration.version, baseline_version) && migration.file_path.exists() {
            fs::remove_file(&migration.file_path).with_context(|| {
                format!(
                    "Failed to delete migration file: {}",
                    migration.file_path.display()
                )
            })?;
            deleted.push(migration.file_path.display().to_string());
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
}
