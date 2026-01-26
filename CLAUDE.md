# migrate

Generic file migration tool for applying ordered transformations to a project directory. Migrations can be written in any language (bash, TypeScript, Python, etc.) using shebangs.

## Commands

```bash
migrate status                              # Show applied/pending migrations
migrate up                                  # Apply all pending migrations
migrate up --dry-run                        # Preview without applying
migrate up --baseline                       # Apply and create baseline at final version
migrate up --baseline --keep                # Apply and baseline without deleting files
migrate create <name>                       # Create new bash migration
migrate create <name> --template ts         # Create TypeScript migration
migrate create <name> --list-templates      # List available templates
migrate baseline <version>                  # Create baseline at specific version
migrate baseline <version> --dry-run        # Preview baseline changes
migrate baseline <version> --keep           # Baseline without deleting files
```

## Options

All commands accept:

- `-r, --root <path>` - Project root (default: `.`)
- `-m, --migrations <path>` - Migrations directory (default: `migrations`)

## Migration Format

Files: `XXXXX-name.{sh,ts,py,js,...}` where XXXXX is a 5-character base36 version.

**Version format:** `DDDMM` where:
- `DDD` = days since 2020-01-01 in base36 (~127 years of runway)
- `MM` = 10-minute slot of the day in base36 (144 slots/day)

Example: `1fb2g-add-config.sh` (created on day 1843 at slot 88 = ~14:40)

Migrations are executable files that receive context via environment variables:

```bash
MIGRATE_PROJECT_ROOT=/path/to/project      # Absolute path to project root
MIGRATE_MIGRATIONS_DIR=/path/to/migrations # Where migration files live
MIGRATE_ID=1fb2g-initial-setup             # Current migration ID (includes version)
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
- `src/version.rs` - Base36 version generation and parsing
- `src/templates.rs` - Embedded migration templates
- `src/baseline.rs` - Baseline management (`.baseline` file)
- `src/commands/` - CLI command implementations
  - `mod.rs` - Command module exports
  - `status.rs` - Status command (shows version summary)
  - `up.rs` - Up command
  - `create.rs` - Create command (generates time-based version)
  - `baseline.rs` - Baseline command (marks versions as applied)
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
