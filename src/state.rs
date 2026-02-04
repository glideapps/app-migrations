use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::{AppliedMigration, Migration};

const HISTORY_FILE: &str = "history";
const LEGACY_HISTORY_FILE: &str = ".history";
const LEGACY_BASELINE_FILE: &str = ".baseline";

/// A baseline assertion: migrations with version <= this are considered applied
#[derive(Debug, Clone)]
pub struct Baseline {
    /// Version string (e.g., "1fb2g")
    pub version: String,
    /// When the baseline was created
    pub created: DateTime<Utc>,
    /// Optional description of what migrations are included
    pub summary: Option<String>,
}

/// State read from the history file
#[derive(Debug, Default)]
pub struct HistoryState {
    pub applied: Vec<AppliedMigration>,
    pub baseline: Option<Baseline>,
}

/// Read the history file and return state (applied migrations and baseline).
/// Handles migration from legacy .history and .baseline files.
pub fn read_history(migrations_dir: &Path) -> Result<HistoryState> {
    let history_path = migrations_dir.join(HISTORY_FILE);
    let legacy_history_path = migrations_dir.join(LEGACY_HISTORY_FILE);
    let legacy_baseline_path = migrations_dir.join(LEGACY_BASELINE_FILE);

    // Migrate legacy files if needed
    if !history_path.exists() && legacy_history_path.exists() {
        migrate_legacy_files(migrations_dir)?;
    }

    if !history_path.exists() {
        return Ok(HistoryState::default());
    }

    let file = fs::File::open(&history_path)
        .with_context(|| format!("Failed to open history file: {}", history_path.display()))?;

    let reader = BufReader::new(file);
    let mut applied = Vec::new();
    let mut baseline: Option<Baseline> = None;

    for line in reader.lines() {
        let line = line.context("Failed to read line from history file")?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        // Baseline format: "baseline: version timestamp [summary]"
        if let Some(rest) = line.strip_prefix("baseline: ") {
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let version = parts[0].to_string();
                let created = DateTime::parse_from_rfc3339(parts[1])
                    .with_context(|| format!("Invalid timestamp in baseline: {}", parts[1]))?
                    .with_timezone(&Utc);
                let summary = if parts.len() == 3 {
                    Some(parts[2].to_string())
                } else {
                    None
                };
                baseline = Some(Baseline {
                    version,
                    created,
                    summary,
                });
            }
            continue;
        }

        // Migration format: "id timestamp" (space-separated)
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

    // Also check for legacy .baseline file that might not have been migrated
    if baseline.is_none() && legacy_baseline_path.exists() {
        if let Some(legacy_baseline) = read_legacy_baseline(&legacy_baseline_path)? {
            // Write it to the new history file and delete the legacy file
            append_baseline(migrations_dir, &legacy_baseline)?;
            fs::remove_file(&legacy_baseline_path).ok();
            baseline = Some(legacy_baseline);
        }
    }

    Ok(HistoryState { applied, baseline })
}

/// Migrate legacy .history and .baseline files to the new history file format.
fn migrate_legacy_files(migrations_dir: &Path) -> Result<()> {
    let history_path = migrations_dir.join(HISTORY_FILE);
    let legacy_history_path = migrations_dir.join(LEGACY_HISTORY_FILE);
    let legacy_baseline_path = migrations_dir.join(LEGACY_BASELINE_FILE);

    // Read legacy history
    let mut content = String::new();
    if legacy_history_path.exists() {
        content = fs::read_to_string(&legacy_history_path).with_context(|| {
            format!(
                "Failed to read legacy history: {}",
                legacy_history_path.display()
            )
        })?;
    }

    // Read and append legacy baseline
    if legacy_baseline_path.exists() {
        if let Some(baseline) = read_legacy_baseline(&legacy_baseline_path)? {
            let baseline_line = format_baseline_line(&baseline);
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&baseline_line);
            content.push('\n');
        }
    }

    // Write new history file
    if !content.is_empty() {
        fs::write(&history_path, &content)
            .with_context(|| format!("Failed to write history file: {}", history_path.display()))?;
    }

    // Remove legacy files
    if legacy_history_path.exists() {
        fs::remove_file(&legacy_history_path).ok();
    }
    if legacy_baseline_path.exists() {
        fs::remove_file(&legacy_baseline_path).ok();
    }

    Ok(())
}

/// Read a legacy .baseline file (YAML-like format)
fn read_legacy_baseline(path: &Path) -> Result<Option<Baseline>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read baseline file: {}", path.display()))?;

    let mut version: Option<String> = None;
    let mut created: Option<DateTime<Utc>> = None;
    let mut summary: Option<String> = None;
    let mut in_summary = false;
    let mut summary_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if in_summary {
            if let Some(stripped) = line.strip_prefix("  ") {
                summary_lines.push(stripped.to_string());
                continue;
            } else if line.starts_with(' ') || line.is_empty() {
                if line.is_empty() {
                    summary_lines.push(String::new());
                } else {
                    summary_lines.push(line.trim_start().to_string());
                }
                continue;
            } else {
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
                in_summary = true;
            } else if !rest.is_empty() {
                summary = Some(rest.to_string());
            }
        }
    }

    if in_summary && !summary_lines.is_empty() {
        summary = Some(summary_lines.join("\n").trim_end().to_string());
    }

    match (version, created) {
        (Some(version), Some(created)) => Ok(Some(Baseline {
            version,
            created,
            summary,
        })),
        _ => Ok(None),
    }
}

/// Format a baseline as a single line for the history file
fn format_baseline_line(baseline: &Baseline) -> String {
    match &baseline.summary {
        Some(summary) => format!(
            "baseline: {} {} {}",
            baseline.version,
            baseline.created.to_rfc3339(),
            summary.replace('\n', " ")
        ),
        None => format!(
            "baseline: {} {}",
            baseline.version,
            baseline.created.to_rfc3339()
        ),
    }
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

/// Append a baseline record to the history file.
pub fn append_baseline(migrations_dir: &Path, baseline: &Baseline) -> Result<()> {
    let history_path = migrations_dir.join(HISTORY_FILE);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)
        .with_context(|| format!("Failed to open history file: {}", history_path.display()))?;

    writeln!(file, "{}", format_baseline_line(baseline))
        .context("Failed to write baseline to history file")?;

    Ok(())
}

/// Get pending migrations (available but not yet applied).
/// If a baseline is provided, skip migrations at or before the baseline version.
pub fn get_pending<'a>(available: &'a [Migration], state: &HistoryState) -> Vec<&'a Migration> {
    let applied_ids: std::collections::HashSet<&str> =
        state.applied.iter().map(|a| a.id.as_str()).collect();

    available
        .iter()
        .filter(|m| {
            // Already applied
            if applied_ids.contains(m.id.as_str()) {
                return false;
            }
            // Covered by baseline (only skip if not in history)
            if let Some(b) = &state.baseline {
                if m.version.as_str() <= b.version.as_str() {
                    return false;
                }
            }
            true
        })
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

        let state = HistoryState {
            applied: vec![AppliedMigration {
                id: "1f700-first".to_string(),
                applied_at: Utc::now(),
            }],
            baseline: None,
        };

        let pending = get_pending(&available, &state);
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].id, "1f710-second");
        assert_eq!(pending[1].id, "1f720-third");
    }

    #[test]
    fn test_get_pending_with_baseline() {
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

        // No applied migrations, but baseline at 1f710
        let state = HistoryState {
            applied: vec![],
            baseline: Some(Baseline {
                version: "1f710".to_string(),
                created: Utc::now(),
                summary: None,
            }),
        };

        let pending = get_pending(&available, &state);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "1f720-third");
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

    #[test]
    fn test_format_baseline_line() {
        let baseline = Baseline {
            version: "1f710".to_string(),
            created: DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            summary: None,
        };
        assert_eq!(
            format_baseline_line(&baseline),
            "baseline: 1f710 2024-06-15T14:30:00+00:00"
        );

        let baseline_with_summary = Baseline {
            version: "1f710".to_string(),
            created: DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            summary: Some("Initial setup\nAdded config".to_string()),
        };
        assert_eq!(
            format_baseline_line(&baseline_with_summary),
            "baseline: 1f710 2024-06-15T14:30:00+00:00 Initial setup Added config"
        );
    }
}
