use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::{AppliedMigration, Migration};

const HISTORY_FILE: &str = ".history";

/// Read the history file and return all applied migrations.
pub fn read_history(migrations_dir: &Path) -> Result<Vec<AppliedMigration>> {
    let history_path = migrations_dir.join(HISTORY_FILE);

    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&history_path)
        .with_context(|| format!("Failed to open history file: {}", history_path.display()))?;

    let reader = BufReader::new(file);
    let mut applied = Vec::new();

    for line in reader.lines() {
        let line = line.context("Failed to read line from history file")?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        // Format: "id timestamp" (space-separated)
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() != 2 {
            continue;
        }

        let id = parts[0].to_string();
        let applied_at = DateTime::parse_from_rfc3339(parts[1])
            .with_context(|| format!("Invalid timestamp in history file: {}", parts[1]))?
            .with_timezone(&Utc);

        applied.push(AppliedMigration { id, applied_at });
    }

    Ok(applied)
}

/// Append a migration record to the history file.
pub fn append_history(migrations_dir: &Path, id: &str, applied_at: DateTime<Utc>) -> Result<()> {
    let history_path = migrations_dir.join(HISTORY_FILE);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)
        .with_context(|| format!("Failed to open history file: {}", history_path.display()))?;

    writeln!(file, "{} {}", id, applied_at.to_rfc3339())
        .context("Failed to write to history file")?;

    Ok(())
}

/// Get pending migrations (available but not yet applied).
pub fn get_pending<'a>(
    available: &'a [Migration],
    applied: &[AppliedMigration],
) -> Vec<&'a Migration> {
    let applied_ids: std::collections::HashSet<&str> =
        applied.iter().map(|a| a.id.as_str()).collect();

    available
        .iter()
        .filter(|m| !applied_ids.contains(m.id.as_str()))
        .collect()
}

/// Get the current version (version of the most recently applied migration).
/// Returns None if no migrations have been applied.
pub fn get_current_version(
    available: &[Migration],
    applied: &[AppliedMigration],
) -> Option<String> {
    // Find the last applied migration that still exists in available
    // (in case a migration was deleted after being applied)
    let applied_ids: std::collections::HashSet<&str> =
        applied.iter().map(|a| a.id.as_str()).collect();

    available
        .iter()
        .rfind(|m| applied_ids.contains(m.id.as_str()))
        .map(|m| m.version.clone())
}

/// Get the target version (version of the latest available migration).
/// Returns None if no migrations are available.
pub fn get_target_version(available: &[Migration]) -> Option<String> {
    available.last().map(|m| m.version.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pending() {
        let available = vec![
            Migration {
                id: "1f700-first".to_string(),
                version: "1f700".to_string(),
                file_path: "1f700-first.sh".into(),
            },
            Migration {
                id: "1f710-second".to_string(),
                version: "1f710".to_string(),
                file_path: "1f710-second.sh".into(),
            },
            Migration {
                id: "1f720-third".to_string(),
                version: "1f720".to_string(),
                file_path: "1f720-third.sh".into(),
            },
        ];

        let applied = vec![AppliedMigration {
            id: "1f700-first".to_string(),
            applied_at: Utc::now(),
        }];

        let pending = get_pending(&available, &applied);
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].id, "1f710-second");
        assert_eq!(pending[1].id, "1f720-third");
    }

    #[test]
    fn test_get_current_version() {
        let available = vec![
            Migration {
                id: "1f700-first".to_string(),
                version: "1f700".to_string(),
                file_path: "1f700-first.sh".into(),
            },
            Migration {
                id: "1f710-second".to_string(),
                version: "1f710".to_string(),
                file_path: "1f710-second.sh".into(),
            },
        ];

        // No applied migrations
        let applied: Vec<AppliedMigration> = vec![];
        assert_eq!(get_current_version(&available, &applied), None);

        // One applied migration
        let applied = vec![AppliedMigration {
            id: "1f700-first".to_string(),
            applied_at: Utc::now(),
        }];
        assert_eq!(
            get_current_version(&available, &applied),
            Some("1f700".to_string())
        );

        // Two applied migrations
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
        assert_eq!(
            get_current_version(&available, &applied),
            Some("1f710".to_string())
        );
    }

    #[test]
    fn test_get_target_version() {
        let available: Vec<Migration> = vec![];
        assert_eq!(get_target_version(&available), None);

        let available = vec![
            Migration {
                id: "1f700-first".to_string(),
                version: "1f700".to_string(),
                file_path: "1f700-first.sh".into(),
            },
            Migration {
                id: "1f710-second".to_string(),
                version: "1f710".to_string(),
                file_path: "1f710-second.sh".into(),
            },
        ];
        assert_eq!(get_target_version(&available), Some("1f710".to_string()));
    }
}
