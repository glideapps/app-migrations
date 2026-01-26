use anyhow::Result;
use std::path::Path;

use crate::baseline::read_baseline;
use crate::loader::discover_migrations;
use crate::state::{get_current_version, get_pending, get_target_version, read_history};

/// Show the status of all migrations
pub fn run(project_root: &Path, migrations_dir: &Path) -> Result<()> {
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

    if available.is_empty() && baseline.is_none() {
        println!("No migrations found in: {}", migrations_path.display());
        return Ok(());
    }

    let current_version = get_current_version(&available, &applied);
    let target_version = get_target_version(&available);

    println!("Migration Status");
    println!("================");
    println!();

    // Show baseline info if present
    if let Some(ref b) = baseline {
        println!("Baseline: {} ({})", b.version, b.created.format("%Y-%m-%d"));
        if let Some(ref summary) = b.summary {
            for line in summary.lines() {
                println!("  {}", line);
            }
        }
        println!();
    }

    // Show version summary line
    match (&current_version, &target_version) {
        (None, Some(target)) if baseline.is_some() => {
            println!(
                "Version: {} -> {} ({} pending)",
                baseline.as_ref().unwrap().version,
                target,
                pending.len()
            );
        }
        (None, Some(target)) => {
            println!("Version: (none) -> {} ({} pending)", target, pending.len());
        }
        (Some(current), Some(target)) if current == target => {
            println!("Version: {} (up to date)", current);
        }
        (Some(current), Some(target)) => {
            println!(
                "Version: {} -> {} ({} pending)",
                current,
                target,
                pending.len()
            );
        }
        (None, None) if baseline.is_some() => {
            println!(
                "Version: {} (up to date, baselined)",
                baseline.as_ref().unwrap().version
            );
        }
        _ => {}
    }
    println!();

    // Show applied migrations
    if !applied.is_empty() {
        println!("Applied ({}):", applied.len());
        for migration in &applied {
            // Check if this migration is at or before baseline
            let is_baselined = baseline
                .as_ref()
                .is_some_and(|b| extract_version(&migration.id) <= Some(b.version.clone()));

            if is_baselined {
                println!(
                    "  + {}  {}  (baseline)",
                    migration.id,
                    migration.applied_at.format("%Y-%m-%d %H:%M:%S")
                );
            } else {
                println!(
                    "  + {}  {}",
                    migration.id,
                    migration.applied_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }
        println!();
    }

    // Show pending migrations
    if !pending.is_empty() {
        println!("Pending ({}):", pending.len());
        for migration in &pending {
            println!("  - {}", migration.id);
        }
    }

    Ok(())
}

/// Extract version from a migration ID (e.g., "1f72f-init" -> "1f72f")
fn extract_version(id: &str) -> Option<String> {
    if id.len() >= 5 && id.chars().nth(5) == Some('-') {
        Some(id[..5].to_string())
    } else {
        None
    }
}
