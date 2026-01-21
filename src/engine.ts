import path from 'path';
import { loadMigrations } from './loader.js';
import { appendHistory, getState } from './state.js';
import {
  createProjectDirectory,
  type MigrationResult,
  type MigrationState,
  type MigrateOptions,
} from './types.js';

/**
 * Get the default migrations directory for a project.
 */
export function getDefaultMigrationsDir(projectRoot: string): string {
  return path.join(projectRoot, 'migrations');
}

/**
 * Get the current migration state for a project.
 */
export async function getMigrationState(
  projectRoot: string,
  options?: { migrationsDir?: string }
): Promise<MigrationState> {
  const migrationsDir = options?.migrationsDir ?? getDefaultMigrationsDir(projectRoot);
  const migrations = await loadMigrations(migrationsDir);
  return getState(migrationsDir, migrations);
}

/**
 * Apply all pending migrations to a project.
 */
export async function migrate(
  projectRoot: string,
  options?: MigrateOptions
): Promise<MigrationResult[]> {
  const migrationsDir = options?.migrationsDir ?? getDefaultMigrationsDir(projectRoot);
  const dryRun = options?.dryRun ?? false;

  // Get current state
  const migrations = await loadMigrations(migrationsDir);
  const state = await getState(migrationsDir, migrations);

  const results: MigrationResult[] = [];

  if (state.pending.length === 0) {
    return results;
  }

  // Create project directory instance
  const project = createProjectDirectory(projectRoot);

  // Apply each pending migration in order
  for (const migration of state.pending) {
    const appliedAt = new Date().toISOString();

    if (dryRun) {
      results.push({
        id: migration.id,
        success: true,
        appliedAt,
        dryRun: true,
      });
      continue;
    }

    try {
      // Execute the migration
      await migration.module.up(project);

      // Record in history
      await appendHistory(migrationsDir, migration.id, appliedAt);

      results.push({
        id: migration.id,
        success: true,
        appliedAt,
        dryRun: false,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      results.push({
        id: migration.id,
        success: false,
        appliedAt,
        error: errorMessage,
        dryRun: false,
      });

      // Stop on first error - don't continue with subsequent migrations
      break;
    }
  }

  return results;
}

/**
 * Create a new migration file.
 */
export async function createMigration(
  migrationsDir: string,
  name: string,
  options?: { description?: string }
): Promise<string> {
  const fs = await import('fs/promises');

  // Ensure migrations directory exists
  await fs.mkdir(migrationsDir, { recursive: true });

  // Find the next prefix number
  const { listMigrationFiles } = await import('./loader.js');
  const existing = await listMigrationFiles(migrationsDir);

  let nextPrefix = 1;
  if (existing.length > 0) {
    const maxPrefix = Math.max(...existing.map((m) => m.prefix));
    nextPrefix = maxPrefix + 1;
  }

  // Format prefix with leading zeros (e.g., 001, 002)
  const prefix = String(nextPrefix).padStart(3, '0');

  // Sanitize name (replace spaces with dashes, remove invalid chars)
  const sanitizedName = name
    .toLowerCase()
    .replace(/\s+/g, '-')
    .replace(/[^a-z0-9-]/g, '');

  const filename = `${prefix}-${sanitizedName}.ts`;
  const filePath = path.join(migrationsDir, filename);

  const description = options?.description ?? `Migration: ${name}`;

  const template = `import fs from 'fs/promises';
import type { ProjectDirectory } from 'app-migrations';

export const description = '${description}';

export async function up(project: ProjectDirectory) {
  // TODO: Implement migration
  // Use project.resolve() to get absolute paths:
  //   const filePath = project.resolve('package.json');
  //
  // Use Node.js fs for file operations:
  //   await fs.writeFile(filePath, content);
  //   await fs.readFile(filePath, 'utf-8');
  //   await fs.mkdir(project.resolve('src'), { recursive: true });
}
`;

  await fs.writeFile(filePath, template, 'utf-8');

  return filePath;
}
