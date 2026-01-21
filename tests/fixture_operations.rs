//! Tests for common migration operations using a fixture directory.
//!
//! These tests verify that migrations can perform typical file operations:
//! - Overwriting files
//! - Editing files (search/replace)
//! - Modifying JSON files
//! - Creating directories and files
//! - Deleting files
//! - Renaming/moving files

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target/debug/migrate");
    path
}

fn get_fixture_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/sample-project");
    path
}

/// Copy the fixture directory to a temp directory for isolated testing
fn setup_fixture() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let fixture_path = get_fixture_path();

    // Recursively copy fixture to temp dir
    copy_dir_all(&fixture_path, temp_dir.path()).expect("Failed to copy fixture");

    // Create migrations directory
    fs::create_dir(temp_dir.path().join("migrations")).expect("Failed to create migrations dir");

    temp_dir
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn create_migration(temp_dir: &Path, name: &str, content: &str) {
    let migrations_dir = temp_dir.join("migrations");
    let migration_path = migrations_dir.join(name);
    fs::write(&migration_path, content).expect("Failed to write migration");

    let mut perms = fs::metadata(&migration_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&migration_path, perms).unwrap();
}

fn run_migrate(temp_dir: &Path) -> std::process::Output {
    Command::new(get_binary_path())
        .args(["--root", temp_dir.to_str().unwrap(), "up"])
        .output()
        .expect("Failed to execute command")
}

// =============================================================================
// Test: Overwrite file
// =============================================================================

#[test]
fn test_migration_overwrites_file() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-overwrite-readme.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

cat > README.md << 'EOF'
# Updated Project

This README has been completely replaced by migration.
EOF
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "Migration should succeed");

    let content = fs::read_to_string(temp_dir.path().join("README.md")).unwrap();
    assert!(content.contains("Updated Project"));
    assert!(content.contains("completely replaced by migration"));
    assert!(!content.contains("Sample Project"));
}

// =============================================================================
// Test: Edit file with sed (search/replace)
// =============================================================================

#[test]
fn test_migration_edits_file_with_sed() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-edit-main.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

# Replace "Hello, world!" with "Hello, migration!"
# Use temp file approach for cross-platform compatibility (BSD vs GNU sed)
sed 's/Hello, world!/Hello, migration!/' src/main.ts > src/main.ts.tmp
mv src/main.ts.tmp src/main.ts
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "Migration should succeed");

    let content = fs::read_to_string(temp_dir.path().join("src/main.ts")).unwrap();
    assert!(content.contains("Hello, migration!"));
    assert!(!content.contains("Hello, world!"));
}

// =============================================================================
// Test: Modify JSON file with jq
// =============================================================================

#[test]
fn test_migration_modifies_json_with_jq() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-update-config.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

# Update version and add a new setting using jq
jq '.version = "2.0.0" | .settings.debug = true | .settings.newFeature = "enabled"' config.json > config.json.tmp
mv config.json.tmp config.json
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    let content = fs::read_to_string(temp_dir.path().join("config.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("Should be valid JSON");

    assert_eq!(json["version"], "2.0.0");
    assert_eq!(json["settings"]["debug"], true);
    assert_eq!(json["settings"]["newFeature"], "enabled");
}

// =============================================================================
// Test: Create new directory and files
// =============================================================================

#[test]
fn test_migration_creates_directory_and_files() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-create-tests.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

mkdir -p tests/unit

cat > tests/unit/main.test.ts << 'EOF'
import { main } from '../../src/main';

describe('main', () => {
  it('should run without error', () => {
    expect(() => main()).not.toThrow();
  });
});
EOF

cat > tests/setup.ts << 'EOF'
// Test setup file
export const testConfig = { timeout: 5000 };
EOF
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "Migration should succeed");

    assert!(temp_dir.path().join("tests/unit").is_dir());
    assert!(temp_dir.path().join("tests/unit/main.test.ts").exists());
    assert!(temp_dir.path().join("tests/setup.ts").exists());

    let test_content = fs::read_to_string(temp_dir.path().join("tests/unit/main.test.ts")).unwrap();
    assert!(test_content.contains("describe('main'"));
}

// =============================================================================
// Test: Delete file
// =============================================================================

#[test]
fn test_migration_deletes_file() {
    let temp_dir = setup_fixture();

    // Verify file exists before migration
    assert!(temp_dir.path().join("data/users.csv").exists());

    create_migration(
        temp_dir.path(),
        "001-delete-users.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

rm data/users.csv
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "Migration should succeed");

    assert!(!temp_dir.path().join("data/users.csv").exists());
}

// =============================================================================
// Test: Rename/move file
// =============================================================================

#[test]
fn test_migration_renames_file() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-rename-config.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

mv config.json app.config.json
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "Migration should succeed");

    assert!(!temp_dir.path().join("config.json").exists());
    assert!(temp_dir.path().join("app.config.json").exists());

    // Verify content is preserved
    let content = fs::read_to_string(temp_dir.path().join("app.config.json")).unwrap();
    assert!(content.contains("sample-project"));
}

// =============================================================================
// Test: Append to file
// =============================================================================

#[test]
fn test_migration_appends_to_file() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-append-csv.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

echo "4,Dave,dave@example.com" >> data/users.csv
echo "5,Eve,eve@example.com" >> data/users.csv
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "Migration should succeed");

    let content = fs::read_to_string(temp_dir.path().join("data/users.csv")).unwrap();
    assert!(content.contains("Alice")); // Original data preserved
    assert!(content.contains("Dave"));
    assert!(content.contains("Eve"));
}

// =============================================================================
// Test: Multiple migrations in sequence
// =============================================================================

#[test]
fn test_multiple_migrations_in_sequence() {
    let temp_dir = setup_fixture();

    // First migration: update version
    create_migration(
        temp_dir.path(),
        "001-bump-version.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

jq '.version = "1.1.0"' config.json > config.json.tmp
mv config.json.tmp config.json
"#,
    );

    // Second migration: add feature flag
    create_migration(
        temp_dir.path(),
        "002-add-feature.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

jq '.features += ["notifications"]' config.json > config.json.tmp
mv config.json.tmp config.json
"#,
    );

    // Third migration: create changelog
    create_migration(
        temp_dir.path(),
        "003-create-changelog.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

cat > CHANGELOG.md << 'EOF'
# Changelog

## 1.1.0
- Added notifications feature
EOF
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "All migrations should succeed");

    // Verify all changes applied
    let config = fs::read_to_string(temp_dir.path().join("config.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&config).unwrap();
    assert_eq!(json["version"], "1.1.0");
    assert!(json["features"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("notifications")));

    assert!(temp_dir.path().join("CHANGELOG.md").exists());

    // Verify history contains all migrations
    let history = fs::read_to_string(temp_dir.path().join("migrations/.history")).unwrap();
    assert!(history.contains("001-bump-version"));
    assert!(history.contains("002-add-feature"));
    assert!(history.contains("003-create-changelog"));
}

// =============================================================================
// Test: TypeScript migration modifies files
// =============================================================================

#[test]
fn test_typescript_migration_modifies_files() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-ts-update-config.ts",
        r#"#!/usr/bin/env -S npx tsx
import * as fs from 'fs/promises';
import * as path from 'path';

const projectRoot = process.env.MIGRATE_PROJECT_ROOT!;

async function main() {
    const configPath = path.join(projectRoot, 'config.json');
    const content = await fs.readFile(configPath, 'utf-8');
    const config = JSON.parse(content);

    config.updatedBy = 'typescript-migration';
    config.settings.tsEnabled = true;

    await fs.writeFile(configPath, JSON.stringify(config, null, 2));
}

main();
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "TypeScript migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    let content = fs::read_to_string(temp_dir.path().join("config.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("Should be valid JSON");

    assert_eq!(json["updatedBy"], "typescript-migration");
    assert_eq!(json["settings"]["tsEnabled"], true);
}

// =============================================================================
// Test: Dry run does not modify fixture files
// =============================================================================

#[test]
fn test_dry_run_preserves_fixture() {
    let temp_dir = setup_fixture();

    let original_readme = fs::read_to_string(temp_dir.path().join("README.md")).unwrap();

    create_migration(
        temp_dir.path(),
        "001-destructive.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
cd "$MIGRATE_PROJECT_ROOT"

rm -rf *
echo "Everything deleted" > DELETED.txt
"#,
    );

    let output = Command::new(get_binary_path())
        .args([
            "--root",
            temp_dir.path().to_str().unwrap(),
            "up",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Dry run should succeed");

    // All original files should still exist
    assert!(temp_dir.path().join("README.md").exists());
    assert!(temp_dir.path().join("config.json").exists());
    assert!(temp_dir.path().join("src/main.ts").exists());
    assert!(!temp_dir.path().join("DELETED.txt").exists());

    // Content should be unchanged
    let current_readme = fs::read_to_string(temp_dir.path().join("README.md")).unwrap();
    assert_eq!(original_readme, current_readme);
}

// =============================================================================
// Test: Migration can read environment variables
// =============================================================================

#[test]
fn test_migration_receives_environment_variables() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-check-env.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail

# Write env vars to a file for verification
cat > "$MIGRATE_PROJECT_ROOT/env-check.txt" << EOF
PROJECT_ROOT=$MIGRATE_PROJECT_ROOT
MIGRATIONS_DIR=$MIGRATE_MIGRATIONS_DIR
MIGRATION_ID=$MIGRATE_ID
DRY_RUN=$MIGRATE_DRY_RUN
EOF
"#,
    );

    let output = run_migrate(temp_dir.path());
    assert!(output.status.success(), "Migration should succeed");

    let env_content = fs::read_to_string(temp_dir.path().join("env-check.txt")).unwrap();

    assert!(env_content.contains("PROJECT_ROOT="));
    assert!(env_content.contains("MIGRATIONS_DIR="));
    assert!(env_content.contains("MIGRATION_ID=001-check-env"));
    assert!(env_content.contains("DRY_RUN=false"));
}

// =============================================================================
// Test: TypeScript AST manipulation to remove deprecated functions
// =============================================================================

#[test]
fn test_typescript_ast_removes_deprecated_functions() {
    let temp_dir = setup_fixture();

    // Migration that uses ts-morph to remove deprecated functions via AST
    create_migration(
        temp_dir.path(),
        "001-remove-deprecated.ts",
        r#"#!/usr/bin/env -S npx tsx
import * as fs from 'fs/promises';
import * as path from 'path';

const projectRoot = process.env.MIGRATE_PROJECT_ROOT!;

async function main() {
    const utilsPath = path.join(projectRoot, 'src/utils.ts');
    let content = await fs.readFile(utilsPath, 'utf-8');

    // Remove functions that contain "deprecated" in their name (case insensitive)
    // This uses regex-based AST-like manipulation for simplicity
    // In a real scenario you'd use ts-morph or typescript compiler API

    // Remove deprecatedHelper function
    content = content.replace(
        /export function deprecatedHelper\(\): void \{[\s\S]*?\n\}\n\n/g,
        ''
    );

    // Remove deprecatedLogger function
    content = content.replace(
        /export function deprecatedLogger\(message: string\): void \{[\s\S]*?\n\}\n\n/g,
        ''
    );

    // Remove DEPRECATED_CONSTANT
    content = content.replace(
        /export const DEPRECATED_CONSTANT = .*;\n\n/g,
        ''
    );

    await fs.writeFile(utilsPath, content);
}

main();
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    let content = fs::read_to_string(temp_dir.path().join("src/utils.ts")).unwrap();

    // Deprecated items should be removed
    assert!(!content.contains("deprecatedHelper"));
    assert!(!content.contains("deprecatedLogger"));
    assert!(!content.contains("DEPRECATED_CONSTANT"));

    // Non-deprecated items should remain
    assert!(content.contains("formatDate"));
    assert!(content.contains("calculateSum"));
    assert!(content.contains("APP_VERSION"));
}

// =============================================================================
// Runtime-specific tests: Verify each supported runtime works
// =============================================================================

#[test]
fn test_bash_runtime_migration() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-bash-test.sh",
        r#"#!/usr/bin/env bash
set -euo pipefail
# Description: Test bash runtime

cd "$MIGRATE_PROJECT_ROOT"

# Create a marker file with bash-specific info
cat > runtime-test-bash.txt << 'EOF'
Runtime: bash
Shell: $BASH_VERSION
EOF

# Append actual shell version
echo "Executed: true" >> runtime-test-bash.txt

# Test bash-specific features: arrays, string manipulation
declare -a files=("config.json" "README.md")
for f in "${files[@]}"; do
    if [[ -f "$f" ]]; then
        echo "Found: $f" >> runtime-test-bash.txt
    fi
done
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Bash migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    let content = fs::read_to_string(temp_dir.path().join("runtime-test-bash.txt")).unwrap();
    assert!(content.contains("Runtime: bash"));
    assert!(content.contains("Executed: true"));
    assert!(content.contains("Found: config.json"));
    assert!(content.contains("Found: README.md"));
}

#[test]
fn test_typescript_runtime_migration() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-typescript-test.ts",
        r#"#!/usr/bin/env -S npx tsx
// Description: Test TypeScript runtime

import * as fs from 'fs/promises';
import * as path from 'path';

const projectRoot = process.env.MIGRATE_PROJECT_ROOT!;
const migrationsDir = process.env.MIGRATE_MIGRATIONS_DIR!;
const migrationId = process.env.MIGRATE_ID!;
const dryRun = process.env.MIGRATE_DRY_RUN === 'true';

interface RuntimeInfo {
    runtime: string;
    nodeVersion: string;
    migrationId: string;
    dryRun: boolean;
    features: string[];
}

async function main() {
    // Test TypeScript-specific features: interfaces, async/await, type annotations
    const info: RuntimeInfo = {
        runtime: 'typescript',
        nodeVersion: process.version,
        migrationId,
        dryRun,
        features: ['async/await', 'interfaces', 'type-annotations', 'es-modules']
    };

    const outputPath = path.join(projectRoot, 'runtime-test-typescript.json');
    await fs.writeFile(outputPath, JSON.stringify(info, null, 2));

    // Also test reading and parsing existing files
    const configPath = path.join(projectRoot, 'config.json');
    const config = JSON.parse(await fs.readFile(configPath, 'utf-8'));

    const verifyPath = path.join(projectRoot, 'runtime-test-typescript-verify.txt');
    await fs.writeFile(verifyPath, `Read config: ${config.name}\nVersion: ${config.version}`);
}

main();
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "TypeScript migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    // Check JSON output
    let json_content =
        fs::read_to_string(temp_dir.path().join("runtime-test-typescript.json")).unwrap();
    let info: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(info["runtime"], "typescript");
    assert_eq!(info["dryRun"], false);
    assert!(info["features"].as_array().unwrap().len() >= 4);

    // Check verification file
    let verify_content =
        fs::read_to_string(temp_dir.path().join("runtime-test-typescript-verify.txt")).unwrap();
    assert!(verify_content.contains("Read config: sample-project"));
}

#[test]
fn test_python_runtime_migration() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-python-test.py",
        r#"#!/usr/bin/env python3
# Description: Test Python runtime

import os
import json
import sys
from pathlib import Path

project_root = Path(os.environ['MIGRATE_PROJECT_ROOT'])
migrations_dir = os.environ['MIGRATE_MIGRATIONS_DIR']
migration_id = os.environ['MIGRATE_ID']
dry_run = os.environ.get('MIGRATE_DRY_RUN', 'false') == 'true'

def main():
    # Test Python-specific features: pathlib, type hints (comment style), json
    info = {
        'runtime': 'python',
        'pythonVersion': f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
        'migrationId': migration_id,
        'dryRun': dry_run,
        'features': ['pathlib', 'f-strings', 'json', 'type-hints']
    }

    output_path = project_root / 'runtime-test-python.json'
    with open(output_path, 'w') as f:
        json.dump(info, f, indent=2)

    # Test reading existing files
    config_path = project_root / 'config.json'
    with open(config_path) as f:
        config = json.load(f)

    verify_path = project_root / 'runtime-test-python-verify.txt'
    with open(verify_path, 'w') as f:
        f.write(f"Read config: {config['name']}\n")
        f.write(f"Version: {config['version']}\n")
        f.write(f"Features count: {len(config.get('features', []))}\n")

if __name__ == '__main__':
    main()
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Python migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    // Check JSON output
    let json_content =
        fs::read_to_string(temp_dir.path().join("runtime-test-python.json")).unwrap();
    let info: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(info["runtime"], "python");
    assert_eq!(info["dryRun"], false);

    // Check verification file
    let verify_content =
        fs::read_to_string(temp_dir.path().join("runtime-test-python-verify.txt")).unwrap();
    assert!(verify_content.contains("Read config: sample-project"));
    assert!(verify_content.contains("Features count: 2"));
}

#[test]
fn test_node_runtime_migration() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-node-test.js",
        r#"#!/usr/bin/env node
// Description: Test Node.js runtime

const fs = require('fs');
const path = require('path');

const projectRoot = process.env.MIGRATE_PROJECT_ROOT;
const migrationsDir = process.env.MIGRATE_MIGRATIONS_DIR;
const migrationId = process.env.MIGRATE_ID;
const dryRun = process.env.MIGRATE_DRY_RUN === 'true';

function main() {
    // Test Node.js-specific features: CommonJS require, sync fs operations
    const info = {
        runtime: 'node',
        nodeVersion: process.version,
        migrationId: migrationId,
        dryRun: dryRun,
        features: ['commonjs', 'require', 'sync-fs', 'process-env']
    };

    const outputPath = path.join(projectRoot, 'runtime-test-node.json');
    fs.writeFileSync(outputPath, JSON.stringify(info, null, 2));

    // Test reading existing files
    const configPath = path.join(projectRoot, 'config.json');
    const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));

    const verifyPath = path.join(projectRoot, 'runtime-test-node-verify.txt');
    fs.writeFileSync(verifyPath,
        `Read config: ${config.name}\n` +
        `Version: ${config.version}\n` +
        `Settings keys: ${Object.keys(config.settings).join(', ')}\n`
    );
}

main();
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Node.js migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    // Check JSON output
    let json_content = fs::read_to_string(temp_dir.path().join("runtime-test-node.json")).unwrap();
    let info: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(info["runtime"], "node");
    assert_eq!(info["dryRun"], false);
    assert!(info["features"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("commonjs")));

    // Check verification file
    let verify_content =
        fs::read_to_string(temp_dir.path().join("runtime-test-node-verify.txt")).unwrap();
    assert!(verify_content.contains("Read config: sample-project"));
    assert!(verify_content.contains("Settings keys: debug, maxRetries"));
}

#[test]
fn test_ruby_runtime_migration() {
    let temp_dir = setup_fixture();

    create_migration(
        temp_dir.path(),
        "001-ruby-test.rb",
        r#"#!/usr/bin/env ruby
# Description: Test Ruby runtime

require 'json'
require 'fileutils'

project_root = ENV['MIGRATE_PROJECT_ROOT']
migrations_dir = ENV['MIGRATE_MIGRATIONS_DIR']
migration_id = ENV['MIGRATE_ID']
dry_run = ENV['MIGRATE_DRY_RUN'] == 'true'

# Test Ruby-specific features: hashes, symbols, JSON, FileUtils
info = {
    runtime: 'ruby',
    rubyVersion: RUBY_VERSION,
    migrationId: migration_id,
    dryRun: dry_run,
    features: ['symbols', 'blocks', 'json', 'fileutils']
}

output_path = File.join(project_root, 'runtime-test-ruby.json')
File.write(output_path, JSON.pretty_generate(info))

# Test reading existing files
config_path = File.join(project_root, 'config.json')
config = JSON.parse(File.read(config_path))

verify_path = File.join(project_root, 'runtime-test-ruby-verify.txt')
File.write(verify_path, <<~VERIFY)
Read config: #{config['name']}
Version: #{config['version']}
Features: #{config['features'].join(', ')}
VERIFY
"#,
    );

    let output = run_migrate(temp_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Ruby migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    // Check JSON output
    let json_content = fs::read_to_string(temp_dir.path().join("runtime-test-ruby.json")).unwrap();
    let info: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(info["runtime"], "ruby");
    assert_eq!(info["dryRun"], false);
    assert!(info["features"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("symbols")));

    // Check verification file
    let verify_content =
        fs::read_to_string(temp_dir.path().join("runtime-test-ruby-verify.txt")).unwrap();
    assert!(verify_content.contains("Read config: sample-project"));
    assert!(verify_content.contains("Features: auth, logging"));
}
