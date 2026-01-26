# migrate

A generic file migration tool that applies ordered transformations to a project directory. Think database migrations, but for files and project setup. Migrations can be written in any language (bash, TypeScript, Python, etc.) using shebangs.

## Install

### Option 1: Download binary (easiest)

Download the pre-built binary for your platform from [GitHub Releases](https://github.com/glideapps/migrate/releases), then move it to a directory in your PATH:

```bash
# Example for macOS (Apple Silicon)
curl -L https://github.com/glideapps/migrate/releases/latest/download/migrate-aarch64-apple-darwin -o migrate
chmod +x migrate
sudo mv migrate /usr/local/bin/
```

### Option 2: cargo-binstall (recommended if you have Rust)

If you have Rust installed, [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) downloads pre-built binaries:

```bash
# Install cargo-binstall first (if you don't have it)
cargo install cargo-binstall

# Then install migrate
cargo binstall migrate
```

### Option 3: cargo install

Requires [Rust](https://rustup.rs):

```bash
cargo install migrate
```

### Option 4: Build from source

```bash
cargo install --git https://github.com/glideapps/migrate
```

## Quick Start

```bash
# Check what migrations exist and their status
migrate status

# Apply all pending migrations
migrate up

# Preview what would happen without making changes
migrate up --dry-run
```

## Migration Lifecycle

### 1. Creating Migrations

Create a new migration with `migrate create`:

```bash
migrate create add-prettier                    # Bash script (default)
migrate create setup-config --template ts      # TypeScript
migrate create init-db --template python       # Python
```

This generates a timestamped file like `1fb2g-add-prettier.sh` in your `migrations/` directory. The 5-character prefix ensures migrations run in chronological order.

**Available templates:** `bash`, `ts`, `python`, `node`, `ruby`

### 2. Writing Migrations

Migrations are executable files that receive context via environment variables:

| Variable | Description |
|----------|-------------|
| `MIGRATE_PROJECT_ROOT` | Absolute path to project root |
| `MIGRATE_MIGRATIONS_DIR` | Where migration files live |
| `MIGRATE_ID` | Current migration ID (e.g., `1fb2g-add-prettier`) |
| `MIGRATE_DRY_RUN` | `true` if running in preview mode |

**Bash example:**

```bash
#!/usr/bin/env bash
set -euo pipefail
# Description: Add TypeScript configuration

cd "$MIGRATE_PROJECT_ROOT"
cat > tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "strict": true
  }
}
EOF
```

**TypeScript example:**

```typescript
#!/usr/bin/env -S npx tsx
// Description: Add configuration file

import * as fs from 'fs/promises';
import * as path from 'path';

const projectRoot = process.env.MIGRATE_PROJECT_ROOT!;

await fs.writeFile(
  path.join(projectRoot, 'config.json'),
  JSON.stringify({ version: 1 }, null, 2)
);
```

### 3. Applying Migrations

Run `migrate up` to apply all pending migrations in order. Each successful migration is recorded in `.history`, so it won't run again.

```bash
migrate up              # Apply all pending
migrate up --dry-run    # Preview without applying
```

If a migration fails, execution stops immediately. Fix the issue and re-run `migrate up`—already-applied migrations are skipped.

### 4. Checking Status

Use `migrate status` to see what's been applied and what's pending:

```
Version: 1fb2g → 1fc3h (2 pending)

Applied (3):
  ✓ 1fa1f-init-project
  ✓ 1fa2g-add-typescript
  ✓ 1fb2g-setup-eslint

Pending (2):
  • 1fc2h-add-prettier
  • 1fc3h-configure-ci
```

### 5. Baselining (Cleaning Up Old Migrations)

Over time, your `migrations/` directory accumulates files. Once migrations have been applied everywhere (all environments, all team members), you can **baseline** to clean up.

Baselining marks a version as the "starting point"—migrations at or before that version are considered complete and can be deleted.

```bash
# Mark version 1fb2g as baseline and delete old migration files
migrate baseline 1fb2g

# Preview what would be deleted without making changes
migrate baseline 1fb2g --dry-run

# Create baseline but keep the files (just update .baseline)
migrate baseline 1fb2g --keep
```

You can also baseline immediately after applying migrations:

```bash
migrate up --baseline           # Apply and baseline at final version
migrate up --baseline --keep    # Apply and baseline without deleting files
```

**When to baseline:**
- All environments have applied the migrations
- All team members have pulled and applied
- You want to reduce clutter in the migrations directory

**What baselining does:**
- Creates/updates `.baseline` file with the baseline version
- Optionally deletes migration files at or before that version
- Future `migrate up` skips migrations covered by the baseline

## Directory Structure

```
your-project/
├── migrations/
│   ├── .history          # Tracks applied migrations (auto-generated)
│   ├── .baseline         # Baseline marker (optional, from baselining)
│   ├── 1fc2h-add-prettier.sh
│   └── 1fc3h-configure-ci.ts
└── ...
```

## Global Options

These options work with all commands:

| Option | Description | Default |
|--------|-------------|---------|
| `-r, --root <path>` | Project root directory | `.` |
| `-m, --migrations <path>` | Migrations directory | `migrations` |

## Development

```bash
git clone https://github.com/glideapps/migrate
cd migrate
./scripts/setup     # Enable git hooks, fetch deps, build, test

cargo build         # Build debug binary
cargo nextest run   # Run tests
cargo fmt           # Format code
cargo clippy        # Lint
```
