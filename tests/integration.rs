use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target/debug/migrate");
    path
}

fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

#[test]
fn test_status_no_migrations_dir() {
    let temp_dir = create_temp_dir();
    let output = Command::new(get_binary_path())
        .args(["--root", temp_dir.path().to_str().unwrap(), "status"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No migrations directory found"));
}

#[test]
fn test_status_empty_migrations_dir() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");
    fs::create_dir(&migrations_dir).unwrap();

    let output = Command::new(get_binary_path())
        .args(["--root", temp_dir.path().to_str().unwrap(), "status"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No migrations found"));
}

#[test]
fn test_create_bash_migration() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");

    let output = Command::new(get_binary_path())
        .args([
            "--root",
            temp_dir.path().to_str().unwrap(),
            "create",
            "test-migration",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Find the created migration file (version is time-based, so we search for pattern)
    let files: Vec<_> = fs::read_dir(&migrations_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with("-test-migration.sh")
        })
        .collect();
    assert_eq!(files.len(), 1, "Migration file should be created");

    let migration_file = files[0].path();

    // Verify filename format: 5 alphanumeric chars + dash + name + extension
    let filename = migration_file.file_name().unwrap().to_string_lossy();
    assert!(
        filename.len() > 6 && filename.chars().take(5).all(|c| c.is_ascii_alphanumeric()),
        "Filename should start with 5-char version: {}",
        filename
    );

    // Check file is executable
    let perms = fs::metadata(&migration_file).unwrap().permissions();
    assert!(perms.mode() & 0o111 != 0, "File should be executable");

    // Check content has shebang
    let content = fs::read_to_string(&migration_file).unwrap();
    assert!(content.starts_with("#!/usr/bin/env bash"));
}

#[test]
fn test_create_typescript_migration() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");

    let output = Command::new(get_binary_path())
        .args([
            "--root",
            temp_dir.path().to_str().unwrap(),
            "create",
            "ts-migration",
            "--template",
            "ts",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Find the created migration file
    let files: Vec<_> = fs::read_dir(&migrations_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with("-ts-migration.ts")
        })
        .collect();
    assert_eq!(files.len(), 1, "Migration file should be created");

    let migration_file = files[0].path();
    let content = fs::read_to_string(&migration_file).unwrap();
    assert!(content.starts_with("#!/usr/bin/env -S npx tsx"));
}

#[test]
fn test_create_detects_version_collision() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");
    fs::create_dir(&migrations_dir).unwrap();

    // Create first migration via CLI
    let output1 = Command::new(get_binary_path())
        .args([
            "--root",
            temp_dir.path().to_str().unwrap(),
            "create",
            "first",
        ])
        .output()
        .expect("Failed to execute command");
    assert!(output1.status.success());

    // Creating second migration immediately should either succeed (different 10-min slot)
    // or fail with version collision message
    let output2 = Command::new(get_binary_path())
        .args([
            "--root",
            temp_dir.path().to_str().unwrap(),
            "create",
            "second",
        ])
        .output()
        .expect("Failed to execute command");

    // If it failed, it should be due to version collision
    if !output2.status.success() {
        let stderr = String::from_utf8_lossy(&output2.stderr);
        assert!(
            stderr.contains("already exists"),
            "Should fail with version collision: {}",
            stderr
        );
    }
}

#[test]
fn test_list_templates() {
    let temp_dir = create_temp_dir();

    let output = Command::new(get_binary_path())
        .args([
            "--root",
            temp_dir.path().to_str().unwrap(),
            "create",
            "dummy",
            "--list-templates",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bash"));
    assert!(stdout.contains("ts"));
    assert!(stdout.contains("python"));
    assert!(stdout.contains("node"));
    assert!(stdout.contains("ruby"));
}

#[test]
fn test_up_applies_migrations() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");
    fs::create_dir(&migrations_dir).unwrap();

    // Create a simple migration with new 5-char version format
    let migration = migrations_dir.join("00001-create-file.sh");
    fs::write(
        &migration,
        r#"#!/usr/bin/env bash
set -euo pipefail
touch "$MIGRATE_PROJECT_ROOT/created-by-migration.txt"
"#,
    )
    .unwrap();

    // Make executable
    let mut perms = fs::metadata(&migration).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&migration, perms).unwrap();

    // Run migrations
    let output = Command::new(get_binary_path())
        .args(["--root", temp_dir.path().to_str().unwrap(), "up"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Migration should succeed: stdout={}, stderr={}",
        stdout,
        stderr
    );

    // Check file was created
    assert!(
        temp_dir.path().join("created-by-migration.txt").exists(),
        "Migration should have created the file"
    );

    // Check history file
    let history = migrations_dir.join(".history");
    assert!(history.exists(), "History file should be created");

    let history_content = fs::read_to_string(&history).unwrap();
    assert!(history_content.contains("00001-create-file"));
}

#[test]
fn test_up_dry_run() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");
    fs::create_dir(&migrations_dir).unwrap();

    // Create a migration with 5-char version format
    let migration = migrations_dir.join("00001-create-file.sh");
    fs::write(
        &migration,
        r#"#!/usr/bin/env bash
touch "$MIGRATE_PROJECT_ROOT/should-not-exist.txt"
"#,
    )
    .unwrap();

    let mut perms = fs::metadata(&migration).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&migration, perms).unwrap();

    // Run with --dry-run
    let output = Command::new(get_binary_path())
        .args([
            "--root",
            temp_dir.path().to_str().unwrap(),
            "up",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dry run"));

    // File should NOT be created
    assert!(
        !temp_dir.path().join("should-not-exist.txt").exists(),
        "Dry run should not execute migration"
    );

    // History should NOT be updated
    assert!(
        !migrations_dir.join(".history").exists(),
        "Dry run should not update history"
    );
}

#[test]
fn test_failed_migration_stops_execution() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");
    fs::create_dir(&migrations_dir).unwrap();

    // First migration succeeds (version 00001)
    let first = migrations_dir.join("00001-success.sh");
    fs::write(
        &first,
        r#"#!/usr/bin/env bash
touch "$MIGRATE_PROJECT_ROOT/first.txt"
"#,
    )
    .unwrap();
    let mut perms = fs::metadata(&first).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&first, perms).unwrap();

    // Second migration fails (version 00002)
    let second = migrations_dir.join("00002-fail.sh");
    fs::write(
        &second,
        r#"#!/usr/bin/env bash
exit 1
"#,
    )
    .unwrap();
    let mut perms = fs::metadata(&second).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&second, perms).unwrap();

    // Third migration should not run (version 00003)
    let third = migrations_dir.join("00003-never.sh");
    fs::write(
        &third,
        r#"#!/usr/bin/env bash
touch "$MIGRATE_PROJECT_ROOT/third.txt"
"#,
    )
    .unwrap();
    let mut perms = fs::metadata(&third).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&third, perms).unwrap();

    // Run migrations
    let output = Command::new(get_binary_path())
        .args(["--root", temp_dir.path().to_str().unwrap(), "up"])
        .output()
        .expect("Failed to execute command");

    // Should fail
    assert!(!output.status.success());

    // First file should exist
    assert!(temp_dir.path().join("first.txt").exists());

    // Third file should NOT exist
    assert!(!temp_dir.path().join("third.txt").exists());

    // History should only contain first migration
    let history = fs::read_to_string(migrations_dir.join(".history")).unwrap();
    assert!(history.contains("00001-success"));
    assert!(!history.contains("00002-fail"));
}

#[test]
fn test_status_shows_applied_and_pending() {
    let temp_dir = create_temp_dir();
    let migrations_dir = temp_dir.path().join("migrations");
    fs::create_dir(&migrations_dir).unwrap();

    // Create migrations with 5-char version format
    let first = migrations_dir.join("00001-first.sh");
    fs::write(&first, "#!/usr/bin/env bash\necho first").unwrap();
    let mut perms = fs::metadata(&first).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&first, perms).unwrap();

    let second = migrations_dir.join("00002-second.sh");
    fs::write(&second, "#!/usr/bin/env bash\necho second").unwrap();
    let mut perms = fs::metadata(&second).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&second, perms).unwrap();

    // Write history for first migration only
    fs::write(
        migrations_dir.join(".history"),
        "00001-first 2024-01-01T00:00:00+00:00\n",
    )
    .unwrap();

    // Check status
    let output = Command::new(get_binary_path())
        .args(["--root", temp_dir.path().to_str().unwrap(), "status"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check for version summary line
    assert!(
        stdout.contains("Version:"),
        "Should show version line: {}",
        stdout
    );
    assert!(stdout.contains("Applied (1)"));
    assert!(stdout.contains("00001-first"));
    assert!(stdout.contains("Pending (1)"));
    assert!(stdout.contains("00002-second"));
}
