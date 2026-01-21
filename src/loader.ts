import { glob } from 'glob';
import path from 'path';
import { pathToFileURL } from 'url';
import type { LoadedMigration, Migration } from './types.js';

/**
 * Pattern for migration filenames: prefix-name.ts (e.g., 001-initial-setup.ts)
 */
const MIGRATION_PATTERN = /^(\d+)-(.+)\.ts$/;

/**
 * Extract the numeric prefix from a migration filename.
 * Returns null if the filename doesn't match the expected pattern.
 */
export function extractPrefix(filename: string): number | null {
  const match = filename.match(MIGRATION_PATTERN);
  if (!match) return null;
  return parseInt(match[1], 10);
}

/**
 * Extract the migration ID from a filename (filename without extension).
 */
export function extractId(filename: string): string {
  return path.basename(filename, '.ts');
}

/**
 * Discover migration files in the given directory.
 * Returns files sorted by their numeric prefix.
 */
export async function discoverMigrations(migrationsDir: string): Promise<string[]> {
  const pattern = path.join(migrationsDir, '[0-9]*-*.ts');
  const files = await glob(pattern, { absolute: true });

  // Filter to only files matching the expected pattern and sort by prefix
  return files
    .filter(file => {
      const filename = path.basename(file);
      return MIGRATION_PATTERN.test(filename);
    })
    .sort((a, b) => {
      const prefixA = extractPrefix(path.basename(a)) ?? 0;
      const prefixB = extractPrefix(path.basename(b)) ?? 0;
      return prefixA - prefixB;
    });
}

/**
 * Load a single migration file and validate its exports.
 */
export async function loadMigrationFile(filePath: string): Promise<LoadedMigration> {
  const filename = path.basename(filePath);
  const prefix = extractPrefix(filename);

  if (prefix === null) {
    throw new Error(`Invalid migration filename: ${filename}. Expected format: NNN-name.ts`);
  }

  const id = extractId(filename);

  // Use file URL for dynamic import (required for ESM)
  const fileUrl = pathToFileURL(filePath).href;
  const module = await import(fileUrl) as Migration;

  // Validate the module has the required exports
  if (typeof module.up !== 'function') {
    throw new Error(`Migration ${id} must export an 'up' function`);
  }

  return {
    id,
    prefix,
    filePath,
    module,
  };
}

/**
 * Discover and load all migrations from the given directory.
 */
export async function loadMigrations(migrationsDir: string): Promise<LoadedMigration[]> {
  const files = await discoverMigrations(migrationsDir);
  const migrations: LoadedMigration[] = [];

  for (const file of files) {
    const migration = await loadMigrationFile(file);
    migrations.push(migration);
  }

  return migrations;
}

/**
 * List all available migrations without loading their modules.
 * Useful for quick status checks.
 */
export async function listMigrationFiles(migrationsDir: string): Promise<{ id: string; prefix: number; filePath: string }[]> {
  const files = await discoverMigrations(migrationsDir);

  return files.map(file => {
    const filename = path.basename(file);
    const prefix = extractPrefix(filename) ?? 0;
    const id = extractId(filename);
    return { id, prefix, filePath: file };
  });
}
