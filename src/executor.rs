use anyhow::{Context, Result};
use std::process::Command;

use crate::{ExecutionContext, ExecutionResult, Migration};

/// Execute a migration file as a subprocess.
/// The migration receives context via environment variables.
pub fn execute(migration: &Migration, ctx: &ExecutionContext) -> Result<ExecutionResult> {
    let status = Command::new(&migration.file_path)
        .env("MIGRATE_PROJECT_ROOT", &ctx.project_root)
        .env("MIGRATE_MIGRATIONS_DIR", &ctx.migrations_dir)
        .env("MIGRATE_ID", &ctx.migration_id)
        .env("MIGRATE_DRY_RUN", ctx.dry_run.to_string())
        .current_dir(&ctx.project_root)
        .status()
        .with_context(|| format!("Failed to execute migration: {}", migration.id))?;

    Ok(ExecutionResult {
        success: status.success(),
        exit_code: status.code().unwrap_or(-1),
        error: if status.success() {
            None
        } else {
            Some(format!(
                "Migration {} failed with exit code {}",
                migration.id,
                status.code().unwrap_or(-1)
            ))
        },
    })
}
