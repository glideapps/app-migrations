use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use migrate::commands;

#[derive(Parser)]
#[command(name = "migrate", version, about = "Generic file migration tool")]
struct Cli {
    /// Project root directory
    #[arg(short = 'r', long, default_value = ".")]
    root: PathBuf,

    /// Migrations directory
    #[arg(short = 'm', long, default_value = "migrations")]
    migrations: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show migration status
    Status,

    /// Apply pending migrations
    Up {
        /// Preview without applying
        #[arg(long)]
        dry_run: bool,

        /// Create baseline at final version after applying (deletes old migration files)
        #[arg(long)]
        baseline: bool,

        /// Keep migration files when using --baseline (don't delete)
        #[arg(long)]
        keep: bool,
    },

    /// Create a new migration
    Create {
        /// Migration name (e.g., "add-config")
        name: Option<String>,

        /// Template to use (bash, ts, python, node, ruby)
        #[arg(short = 't', long, default_value = "bash")]
        template: String,

        /// Migration description
        #[arg(short = 'd', long)]
        description: Option<String>,

        /// List available templates
        #[arg(long)]
        list_templates: bool,
    },

    /// Create a baseline at a specific version (removes old migration files)
    Baseline {
        /// Version to baseline at (e.g., "1fb2g")
        version: String,

        /// Summary description for the baseline
        #[arg(short = 's', long)]
        summary: Option<String>,

        /// Preview without making changes
        #[arg(long)]
        dry_run: bool,

        /// Keep migration files (don't delete)
        #[arg(long)]
        keep: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            commands::status::run(&cli.root, &cli.migrations)?;
        }
        Commands::Up {
            dry_run,
            baseline,
            keep,
        } => {
            commands::up::run(&cli.root, &cli.migrations, dry_run, baseline, keep)?;
        }
        Commands::Create {
            name,
            template,
            description,
            list_templates,
        } => {
            commands::create::run(
                &cli.root,
                &cli.migrations,
                name.as_deref(),
                &template,
                description.as_deref(),
                list_templates,
            )?;
        }
        Commands::Baseline {
            version,
            summary,
            dry_run,
            keep,
        } => {
            commands::baseline::run(
                &cli.root,
                &cli.migrations,
                &version,
                summary.as_deref(),
                dry_run,
                keep,
            )?;
        }
    }

    Ok(())
}
