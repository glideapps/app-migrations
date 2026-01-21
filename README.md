# app-migrations

A file system migration tool that applies ordered transformations to a project directory. Think database migrations, but for files and project setup.

## Installation

```bash
npm install app-migrations
```

## Usage

### Check migration status

```bash
app-migrate status
```

### Apply pending migrations

```bash
app-migrate up
```

### Preview changes without applying

```bash
app-migrate up --dry-run
```

### Create a new migration

```bash
app-migrate create add-prettier -d "Add Prettier configuration"
```

## Writing Migrations

Create files in your migrations directory with the pattern `NNN-name.ts`:

```typescript
// migrations/001-add-typescript.ts
import type { ProjectDirectory } from 'app-migrations';
import * as fs from 'node:fs/promises';

export const description = 'Add TypeScript configuration';

export async function up(project: ProjectDirectory): Promise<void> {
  const tsconfig = {
    compilerOptions: {
      target: 'ES2022',
      module: 'NodeNext',
      strict: true,
    },
  };

  await fs.writeFile(project.resolve('tsconfig.json'), JSON.stringify(tsconfig, null, 2));
}
```

Migrations run in order by their numeric prefix and are tracked in a `.history` file.

## CLI Reference

| Command                     | Description                         |
| --------------------------- | ----------------------------------- |
| `app-migrate status`        | Show applied and pending migrations |
| `app-migrate up`            | Apply all pending migrations        |
| `app-migrate create <name>` | Create a new migration file         |

### Options

| Option                     | Description                         | Default      |
| -------------------------- | ----------------------------------- | ------------ |
| `-r, --root <path>`        | Project root directory              | `.`          |
| `-m, --migrations <path>`  | Migrations directory                | `migrations` |
| `--dry-run`                | Preview changes (up only)           | `false`      |
| `-d, --description <text>` | Migration description (create only) | -            |
