pub mod baseline;
pub mod commands;
pub mod executor;
pub mod loader;
pub mod state;
pub mod templates;
pub mod version;

use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Metadata for a discovered migration file
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration ID (e.g., "1f72f-init")
    pub id: String,
    /// Version string (e.g., "1f72f")
    pub version: String,
    /// Absolute path to the migration file
    pub file_path: PathBuf,
}

/// Record of an applied migration
#[derive(Debug, Clone)]
pub struct AppliedMigration {
    /// Migration ID
    pub id: String,
    /// When the migration was applied
    pub applied_at: DateTime<Utc>,
}

/// Execution context passed via environment variables
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Absolute path to project root
    pub project_root: PathBuf,
    /// Path to migrations directory
    pub migrations_dir: PathBuf,
    /// Current migration ID
    pub migration_id: String,
    /// Whether this is a dry run
    pub dry_run: bool,
}

/// Result of executing a migration
#[derive(Debug)]
pub struct ExecutionResult {
    /// Whether the migration succeeded
    pub success: bool,
    /// Exit code from the subprocess
    pub exit_code: i32,
    /// Error message if any
    pub error: Option<String>,
}
