use anyhow::{bail, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::loader::discover_migrations;
use crate::templates::{get_template, list_templates};

/// Create a new migration file
pub fn run(
    project_root: &Path,
    migrations_dir: &Path,
    name: Option<&str>,
    template_name: &str,
    description: Option<&str>,
    should_list_templates: bool,
) -> Result<()> {
    // Handle --list-templates flag
    if should_list_templates {
        println!("Available templates:");
        for template in list_templates() {
            println!("  {}", template);
        }
        return Ok(());
    }

    // Name is required when not listing templates
    let name = match name {
        Some(n) => n,
        None => bail!("Migration name is required. Usage: migrate create <name>"),
    };

    // Validate template
    let template = match get_template(template_name) {
        Some(t) => t,
        None => {
            bail!(
                "Unknown template '{}'. Available: {}",
                template_name,
                list_templates().collect::<Vec<_>>().join(", ")
            );
        }
    };

    let migrations_path = if migrations_dir.is_absolute() {
        migrations_dir.to_path_buf()
    } else {
        project_root.join(migrations_dir)
    };

    // Create migrations directory if it doesn't exist
    fs::create_dir_all(&migrations_path)?;

    // Determine next prefix
    let existing = discover_migrations(&migrations_path).unwrap_or_default();
    let next_prefix = existing.iter().map(|m| m.prefix).max().unwrap_or(0) + 1;

    // Build filename
    let filename = format!("{:03}-{}{}", next_prefix, name, template.extension);
    let file_path = migrations_path.join(&filename);

    // Check if file already exists
    if file_path.exists() {
        bail!("Migration file already exists: {}", file_path.display());
    }

    // Prepare template content
    let description_text = description.unwrap_or("TODO: Add description");
    let content = template
        .content
        .replace("{{DESCRIPTION}}", description_text);

    // Write file
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file_path)?;

    file.write_all(content.as_bytes())?;

    // Make executable (chmod +x)
    let mut perms = fs::metadata(&file_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&file_path, perms)?;

    println!("Created migration: {}", file_path.display());

    Ok(())
}
