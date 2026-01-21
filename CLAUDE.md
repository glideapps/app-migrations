# app-migrations

File system migration tool for applying ordered transformations to a project directory.

## Commands

```bash
app-migrate status              # Show applied/pending migrations
app-migrate up                  # Apply all pending migrations
app-migrate up --dry-run        # Preview without applying
app-migrate create <name>       # Create new migration file
```

## Options

All commands accept:

- `-r, --root <path>` - Project root (default: `.`)
- `-m, --migrations <path>` - Migrations directory (default: `migrations`)

## Migration Format

Files: `NNN-name.ts` (e.g., `001-add-typescript.ts`)

```typescript
import type { ProjectDirectory } from 'app-migrations';

export const description = 'What this migration does';

export async function up(project: ProjectDirectory): Promise<void> {
  // project.resolve('path') returns absolute path within project
}
```

## Architecture

- `src/cli.ts` - CLI commands
- `src/engine.ts` - Migration execution
- `src/loader.ts` - Discovery and loading
- `src/state.ts` - History tracking (`.history` file)
- `src/types.ts` - TypeScript interfaces

## Development

```bash
npm test          # Run tests
npm run build     # Build
npm run lint      # Lint
```
