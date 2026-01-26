use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::baseline::{delete_baselined_migrations, read_baseline, write_baseline, Baseline};
use crate::executor::execute;
use crate::loader::discover_migrations;
use crate::state::{append_history, get_pending, read_history};
use crate::ExecutionContext;

/// Apply all pending migrations
pub fn run(
    project_root: &Path,
    migrations_dir: &Path,
    dry_run: bool,
    create_baseline: bool,
    keep: bool,
) -> Result<()> {
    let project_root = if project_root.is_absolute() {
        project_root.to_path_buf()
    } else {
        std::env::current_dir()?.join(project_root)
    };

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
    let baseline = read_baseline(&migrations_path)?;
    let pending = get_pending(&available, &applied, baseline.as_ref());

    if pending.is_empty() {
        println!("No pending migrations.");
        return Ok(());
    }

    println!(
        "{} {} migration(s)...",
        if dry_run { "Would apply" } else { "Applying" },
        pending.len()
    );
    println!();

    let mut last_applied_version: Option<String> = None;

    for migration in &pending {
        println!("→ {}", migration.id);

        if dry_run {
            println!("  (dry run - skipped)");
            last_applied_version = Some(migration.version.clone());
            continue;
        }

        let ctx = ExecutionContext {
            project_root: project_root.clone(),
            migrations_dir: migrations_path.clone(),
            migration_id: migration.id.clone(),
            dry_run,
        };

        let result = execute(migration, &ctx)?;

        if result.success {
            let applied_at = Utc::now();
            append_history(&migrations_path, &migration.id, applied_at)?;
            last_applied_version = Some(migration.version.clone());
            println!("  ✓ completed");
        } else {
            println!("  ✗ failed (exit code {})", result.exit_code);
            if let Some(error) = result.error {
                println!("    {}", error);
            }
            return Err(anyhow::anyhow!(
                "Migration {} failed with exit code {}",
                migration.id,
                result.exit_code
            ));
        }
    }

    println!();
    println!("All migrations applied successfully.");

    // Handle --baseline flag
    if create_baseline {
        if let Some(version) = last_applied_version {
            println!();
            if dry_run {
                println!("Would create baseline at version '{}'", version);
                if !keep {
                    let to_delete: Vec<_> = available
                        .iter()
                        .filter(|m| m.version.as_str() <= version.as_str())
                        .collect();
                    if !to_delete.is_empty() {
                        println!("Would delete {} migration file(s)", to_delete.len());
                    }
                }
            } else {
                let new_baseline = Baseline {
                    version: version.clone(),
                    created: Utc::now(),
                    summary: None,
                };

                write_baseline(&migrations_path, &new_baseline)?;
                println!("Created baseline at version '{}'", version);

                if !keep {
                    let deleted = delete_baselined_migrations(&version, &available)?;
                    if !deleted.is_empty() {
                        println!("Deleted {} migration file(s)", deleted.len());
                    }
                }
            }
        }
    }

    Ok(())
}
