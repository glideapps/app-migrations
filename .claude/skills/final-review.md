# Final Review Skill

When performing a final review before merge, include the following checks:

## Template Parity Check

Ensure all supported templates have parity across:

1. **Template files** (`templates/` directory):
   - Each template should have a file (e.g., `bash.sh`, `typescript.ts`, `python.py`, `node.js`, `ruby.rb`)
   - All templates should include the same example operations (copy file, update JSON, replace directory)

2. **Template registration** (`src/templates.rs`):
   - Each template file should be registered in the `TEMPLATES` array
   - Verify correct extension mapping

3. **CLI help text** (`src/main.rs`):
   - The `--template` flag help text should list all supported templates

4. **Documentation** (`CLAUDE.md`):
   - The templates section should list all template files

5. **Integration tests** (`tests/integration.rs`):
   - The `test_list_templates` test should assert all templates are present

6. **Fixture tests** (`tests/fixture_operations.rs`):
   - Each template should have a dedicated runtime test (e.g., `test_bash_runtime_migration`, `test_ruby_runtime_migration`)

## CLI Command and Option Coverage

Verify each CLI command and option is documented and tested:

### Commands

| Command | Documentation | Integration Test | Fixture Test |
|---------|---------------|------------------|--------------|
| `status` | CLAUDE.md | `test_status_no_migrations_dir`, `test_status_empty_migrations_dir`, `test_status_shows_applied_and_pending` | - |
| `up` | CLAUDE.md | `test_up_applies_migrations`, `test_failed_migration_stops_execution` | All `test_migration_*` tests |
| `create` | CLAUDE.md | `test_create_bash_migration`, `test_create_typescript_migration`, `test_create_increments_prefix` | - |

### Global Options

| Option | Documentation | Integration Test |
|--------|---------------|------------------|
| `-r, --root` | CLAUDE.md | Used in all integration tests |
| `-m, --migrations` | CLAUDE.md | Default used in tests |

### `up` Options

| Option | Documentation | Integration Test | Fixture Test |
|--------|---------------|------------------|--------------|
| `--dry-run` | CLAUDE.md | `test_up_dry_run` | `test_dry_run_preserves_fixture` |

### `create` Options

| Option | Documentation | Integration Test |
|--------|---------------|------------------|
| `--template` | CLAUDE.md | `test_create_typescript_migration` |
| `--description` | CLAUDE.md | (implicit in template content) |
| `--list-templates` | CLAUDE.md | `test_list_templates` |

## How to verify template parity

Run this command to list all templates from the CLI:
```bash
cargo run -- create dummy --list-templates
```

Then verify each template appears in:
- `templates/` directory (one file per template)
- `src/templates.rs` TEMPLATES array
- `src/main.rs` help text for `--template`
- `CLAUDE.md` templates section
- `tests/integration.rs` test_list_templates assertions
- `tests/fixture_operations.rs` runtime test functions

## How to verify CLI coverage

1. Run `cargo run -- -h` and `cargo run -- <command> -h` for each command
2. Verify each option shown in help output has:
   - A corresponding entry in CLAUDE.md
   - At least one integration test that exercises it
   - A fixture test for options that affect migration execution

## Checklist

- [ ] All templates have parity (same example operations)
- [ ] All CLI commands documented in CLAUDE.md
- [ ] All CLI options documented in CLAUDE.md
- [ ] Each command has integration tests
- [ ] Each option has at least one test
- [ ] Migration execution options have fixture tests
- [ ] `cargo nextest run` passes
- [ ] `cargo clippy` passes
- [ ] `cargo fmt --check` passes
