# migrate

A generic file migration tool that applies ordered transformations to a project directory. Think database migrations, but for files and project setup. Migrations can be written in any language (bash, TypeScript, Python, etc.) using shebangs.

## Installation

### Pre-built binaries (recommended)

Using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall migrate
```

Or download directly from [GitHub Releases](https://github.com/glideapps/migrate/releases).

### From source

```bash
# Install from GitHub (requires Rust: https://rustup.rs)
cargo install --git https://github.com/glideapps/migrate

# Or clone and build locally
git clone https://github.com/glideapps/migrate
cd migrate
cargo install --path .
```

## Usage

### Check migration status

```bash
migrate status
```

### Apply pending migrations

```bash
migrate up
```

### Preview changes without applying

```bash
migrate up --dry-run
```

### Create a new migration

```bash
# Create a bash migration (default)
migrate create add-prettier

# Create a TypeScript migration
migrate create add-config --template ts

# Create with description
migrate create add-prettier -d "Add Prettier configuration"

# List available templates
migrate create --list-templates
```

## Writing Migrations

Migrations are executable files that receive context via environment variables:

```bash
MIGRATE_PROJECT_ROOT=/path/to/project      # Absolute path to project root
MIGRATE_MIGRATIONS_DIR=/path/to/migrations # Where migration files live
MIGRATE_ID=001-initial-setup               # Current migration ID
MIGRATE_DRY_RUN=true|false                 # Whether this is a dry run
```

### Bash example

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

### TypeScript example

```typescript
#!/usr/bin/env -S npx tsx
// Description: Add configuration file

import * as fs from 'fs/promises';
import * as path from 'path';

const projectRoot = process.env.MIGRATE_PROJECT_ROOT!;

const config = {
  version: 1,
  features: ['auth', 'api']
};

await fs.writeFile(
  path.join(projectRoot, 'config.json'),
  JSON.stringify(config, null, 2)
);
```

Migrations run in order by their numeric prefix (e.g., `001-`, `002-`) and are tracked in a `.history` file.

## CLI Reference

| Command                     | Description                         |
| --------------------------- | ----------------------------------- |
| `migrate status`            | Show applied and pending migrations |
| `migrate up`                | Apply all pending migrations        |
| `migrate create <name>`     | Create a new migration file         |

### Options

| Option                     | Description                           | Default      |
| -------------------------- | ------------------------------------- | ------------ |
| `-r, --root <path>`        | Project root directory                | `.`          |
| `-m, --migrations <path>`  | Migrations directory                  | `migrations` |
| `--dry-run`                | Preview changes (up only)             | `false`      |
| `-t, --template <name>`    | Template to use (create only)         | `bash`       |
| `-d, --description <text>` | Migration description (create only)   | -            |
| `--list-templates`         | List available templates (create only)| -            |

## Available Templates

- `bash` - Shell script (`.sh`)
- `ts` - TypeScript via tsx (`.ts`)
- `python` - Python 3 (`.py`)
- `node` - Node.js (`.js`)
- `ruby` - Ruby (`.rb`)

## Development

```bash
# Clone and setup
git clone <repo-url>
cd migrate
./scripts/setup     # Enable git hooks, fetch deps, build, test

# Common commands
cargo build         # Build debug binary
cargo nextest run   # Run tests
cargo fmt           # Format code
cargo clippy        # Lint
cargo run -- status # Run CLI locally

# Build release
cargo build --release
```
