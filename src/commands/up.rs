use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::executor::execute;
use crate::loader::discover_migrations;
use crate::state::{append_history, get_pending, read_history};
use crate::ExecutionContext;

/// Apply all pending migrations
pub fn run(project_root: &Path, migrations_dir: &Path, dry_run: bool) -> Result<()> {
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
    let pending = get_pending(&available, &applied);

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

    for migration in pending {
        println!("→ {}", migration.id);

        if dry_run {
            println!("  (dry run - skipped)");
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

    Ok(())
}
