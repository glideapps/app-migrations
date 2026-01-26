use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::baseline::{
    delete_baselined_migrations, read_baseline, validate_baseline, write_baseline, Baseline,
};

use crate::loader::discover_migrations;
use crate::state::read_history;

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
    let applied = read_history(&migrations_path)?;
    let existing_baseline = read_baseline(&migrations_path)?;

    // Validate the baseline
    validate_baseline(version, &available, &applied, existing_baseline.as_ref())?;

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
        println!(
            "{} migration file(s) to delete:",
            if dry_run { "Would delete" } else { "Deleting" }
        );
        for migration in &to_delete {
            println!("  - {}", migration.id);
        }
        println!();
    } else if keep {
        println!("Keeping migration files (--keep flag)");
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

    write_baseline(&migrations_path, &baseline)?;
    println!("Created .baseline file");

    // Delete old migration files unless --keep was specified
    if !keep && !to_delete.is_empty() {
        let deleted = delete_baselined_migrations(version, &available)?;
        println!("Deleted {} migration file(s)", deleted.len());
    }

    println!();
    println!("Baseline created successfully at version '{}'", version);

    Ok(())
}
