# migrate

Generic file migration tool for applying ordered transformations to a project directory. Migrations can be written in any language (bash, TypeScript, Python, etc.) using shebangs.

## Commands

```bash
migrate status                              # Show applied/pending migrations
migrate up                                  # Apply all pending migrations
migrate up --dry-run                        # Preview without applying
migrate create <name>                       # Create new bash migration
migrate create <name> --template ts         # Create TypeScript migration
migrate create <name> --list-templates      # List available templates
```

## Options

All commands accept:

- `-r, --root <path>` - Project root (default: `.`)
- `-m, --migrations <path>` - Migrations directory (default: `migrations`)

## Migration Format

Files: `NNN-name.{sh,ts,py,js,...}` (e.g., `001-add-config.sh`)

Migrations are executable files that receive context via environment variables:

```bash
MIGRATE_PROJECT_ROOT=/path/to/project      # Absolute path to project root
MIGRATE_MIGRATIONS_DIR=/path/to/migrations # Where migration files live
MIGRATE_ID=001-initial-setup               # Current migration ID
MIGRATE_DRY_RUN=true|false                 # Whether this is a dry run
```

**Bash example:**
```bash
#!/usr/bin/env bash
set -euo pipefail
# Description: Initialize project structure

cd "$MIGRATE_PROJECT_ROOT"
mkdir -p config
```

**TypeScript example:**
```typescript
#!/usr/bin/env -S npx tsx
// Description: Add configuration file

import * as fs from 'fs/promises';
const projectRoot = process.env.MIGRATE_PROJECT_ROOT!;
await fs.writeFile(`${projectRoot}/config.json`, '{}');
```

## Architecture

- `src/main.rs` - CLI entry point (clap)
- `src/lib.rs` - Core types and public API
- `src/loader.rs` - Migration discovery
- `src/executor.rs` - Subprocess execution
- `src/state.rs` - History tracking (`.history` file)
- `src/templates.rs` - Embedded migration templates
- `src/commands/` - CLI command implementations
  - `mod.rs` - Command module exports
  - `status.rs` - Status command
  - `up.rs` - Up command
  - `create.rs` - Create command
- `templates/` - Template source files (bash.sh, typescript.ts, python.py, node.js, ruby.rb)

## Development

```bash
# Setup (enable git hooks, verify toolchain)
./scripts/setup

# Build and test
cargo nextest run      # Run tests
cargo build            # Build debug binary
cargo build --release  # Build release binary
cargo fmt              # Format code
cargo clippy           # Lint
```
