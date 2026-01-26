use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

const BASELINE_FILE: &str = ".baseline";

/// A baseline assertion: migrations with version <= this are no longer required as files
#[derive(Debug, Clone)]
pub struct Baseline {
    /// Version string (e.g., "1fb2g")
    pub version: String,
    /// When the baseline was created
    pub created: DateTime<Utc>,
    /// Optional description of what migrations are included
    pub summary: Option<String>,
}

/// Read the baseline file if it exists.
pub fn read_baseline(migrations_dir: &Path) -> Result<Option<Baseline>> {
    let baseline_path = migrations_dir.join(BASELINE_FILE);

    if !baseline_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&baseline_path)
        .with_context(|| format!("Failed to read baseline file: {}", baseline_path.display()))?;

    parse_baseline(&content).map(Some)
}

/// Write the baseline file.
pub fn write_baseline(migrations_dir: &Path, baseline: &Baseline) -> Result<()> {
    let baseline_path = migrations_dir.join(BASELINE_FILE);

    let mut content = format!(
        "version: {}\ncreated: {}\n",
        baseline.version,
        baseline.created.to_rfc3339()
    );

    if let Some(summary) = &baseline.summary {
        content.push_str("summary: |\n");
        for line in summary.lines() {
            content.push_str("  ");
            content.push_str(line);
            content.push('\n');
        }
    }

    fs::write(&baseline_path, content)
        .with_context(|| format!("Failed to write baseline file: {}", baseline_path.display()))?;

    Ok(())
}

/// Parse baseline file content into a Baseline struct.
fn parse_baseline(content: &str) -> Result<Baseline> {
    let mut version: Option<String> = None;
    let mut created: Option<DateTime<Utc>> = None;
    let mut summary: Option<String> = None;
    let mut in_summary = false;
    let mut summary_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if in_summary {
            // Summary lines are indented with 2 spaces
            if let Some(stripped) = line.strip_prefix("  ") {
                summary_lines.push(stripped.to_string());
                continue;
            } else if line.starts_with(' ') || line.is_empty() {
                // Still in summary block
                if line.is_empty() {
                    summary_lines.push(String::new());
                } else {
                    summary_lines.push(line.trim_start().to_string());
                }
                continue;
            } else {
                // End of summary block
                in_summary = false;
                summary = Some(summary_lines.join("\n").trim_end().to_string());
                summary_lines.clear();
            }
        }

        if let Some(stripped) = line.strip_prefix("version:") {
            version = Some(stripped.trim().to_string());
        } else if let Some(stripped) = line.strip_prefix("created:") {
            let timestamp_str = stripped.trim();
            created = Some(
                DateTime::parse_from_rfc3339(timestamp_str)
                    .with_context(|| format!("Invalid timestamp in baseline: {}", timestamp_str))?
                    .with_timezone(&Utc),
            );
        } else if let Some(stripped) = line.strip_prefix("summary:") {
            let rest = stripped.trim();
            if rest == "|" {
                // Multi-line summary
                in_summary = true;
            } else if !rest.is_empty() {
                // Single-line summary
                summary = Some(rest.to_string());
            }
        }
    }

    // Handle summary at end of file
    if in_summary && !summary_lines.is_empty() {
        summary = Some(summary_lines.join("\n").trim_end().to_string());
    }

    let version = version.context("Baseline file missing 'version' field")?;
    let created = created.context("Baseline file missing 'created' field")?;

    Ok(Baseline {
        version,
        created,
        summary,
    })
}

/// Compare two version strings. Returns true if v1 <= v2.
pub fn version_lte(v1: &str, v2: &str) -> bool {
    v1 <= v2
}

/// Delete migration files at or before the baseline version.
/// Returns the list of deleted file paths.
pub fn delete_baselined_migrations(
    baseline_version: &str,
    available: &[crate::Migration],
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
    available: &[crate::Migration],
    applied: &[crate::AppliedMigration],
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
    use crate::{AppliedMigration, Migration};
    use std::path::PathBuf;

    #[test]
    fn test_parse_baseline_simple() {
        let content = "version: 1fb2g\ncreated: 2024-06-15T14:30:00Z\n";
        let baseline = parse_baseline(content).unwrap();
        assert_eq!(baseline.version, "1fb2g");
        assert!(baseline.summary.is_none());
    }

    #[test]
    fn test_parse_baseline_with_summary() {
        let content = r#"version: 1fb2g
created: 2024-06-15T14:30:00Z
summary: |
  Initial project setup
  TypeScript config
"#;
        let baseline = parse_baseline(content).unwrap();
        assert_eq!(baseline.version, "1fb2g");
        assert_eq!(
            baseline.summary,
            Some("Initial project setup\nTypeScript config".to_string())
        );
    }

    #[test]
    fn test_parse_baseline_single_line_summary() {
        let content = "version: 1fb2g\ncreated: 2024-06-15T14:30:00Z\nsummary: Initial setup\n";
        let baseline = parse_baseline(content).unwrap();
        assert_eq!(baseline.version, "1fb2g");
        assert_eq!(baseline.summary, Some("Initial setup".to_string()));
    }

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
