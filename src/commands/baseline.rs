use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::baseline::{delete_baselined_migrations, validate_baseline, DeletedItem};
use crate::loader::discover_migrations;
use crate::state::{append_baseline, read_history, Baseline};

/// Create a baseline at the specified version
pub fn run(
    project_root: &Path,
    migrations_dir: &Path,
    version: &str,
    summary: Option<&str>,
    dry_run: bool,
    keep: bool,
) -> Result<()> {
    let migrations_path = if migrations_dir.is_absolute() {
        migrations_dir.to_path_buf()
    } else {
        project_root.join(migrations_dir)
    };

    if !migrations_path.exists() {
        println!(
            "No migrations directory found at: {}",
            migrations_path.display()
        );
        return Ok(());
    }

    let available = discover_migrations(&migrations_path)?;
    let state = read_history(&migrations_path)?;

    // Validate the baseline
    validate_baseline(version, &available, &state.applied, state.baseline.as_ref())?;

    // Find migrations that would be deleted
    let to_delete: Vec<_> = available
        .iter()
        .filter(|m| m.version.as_str() <= version)
        .collect();

    if dry_run {
        println!("Dry run - no changes will be made");
        println!();
    }

    println!(
        "Creating baseline at version '{}'{}",
        version,
        if dry_run { " (dry run)" } else { "" }
    );
    println!();

    if !to_delete.is_empty() && !keep {
        println!("{}:", if dry_run { "Would delete" } else { "Deleting" });
        for migration in &to_delete {
            let asset_dir_exists = migration
                .file_path
                .parent()
                .map(|p| p.join(&migration.id).is_dir())
                .unwrap_or(false);
            if asset_dir_exists {
                println!("  - {} (file + {}/)", migration.id, migration.id);
            } else {
                println!("  - {}", migration.id);
            }
        }
        println!();
    } else if keep {
        let has_any_asset_dir = to_delete.iter().any(|m| {
            m.file_path
                .parent()
                .map(|p| p.join(&m.id).is_dir())
                .unwrap_or(false)
        });
        if has_any_asset_dir {
            println!("Keeping migration files and asset directories (--keep flag)");
        } else {
            println!("Keeping migration files (--keep flag)");
        }
        println!();
    }

    if dry_run {
        return Ok(());
    }

    // Create the baseline
    let baseline = Baseline {
        version: version.to_string(),
        created: Utc::now(),
        summary: summary.map(|s| s.to_string()),
    };

    append_baseline(&migrations_path, &baseline)?;
    println!("Added baseline to history file");

    // Delete old migration files and asset directories unless --keep was specified
    if !keep && !to_delete.is_empty() {
        let deleted = delete_baselined_migrations(version, &available)?;
        let (files, dirs): (Vec<&DeletedItem>, Vec<&DeletedItem>) =
            deleted.iter().partition(|d| !d.is_directory);
        if !files.is_empty() {
            println!("Deleted {} migration file(s)", files.len());
        }
        if !dirs.is_empty() {
            println!("Deleted {} asset directory(ies)", dirs.len());
        }
    }

    println!();
    println!("Baseline created successfully at version '{}'", version);

    Ok(())
}
