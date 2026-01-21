import fs from 'fs/promises';
import path from 'path';
import type { AppliedMigration, LoadedMigration, MigrationState } from './types.js';

const HISTORY_FILE = '.history';

/**
 * Get the path to the history file for a migrations directory.
 */
export function getHistoryPath(migrationsDir: string): string {
  return path.join(migrationsDir, HISTORY_FILE);
}

/**
 * Read the history file and return a list of applied migrations.
 * Returns an empty array if the history file doesn't exist.
 */
export async function readHistory(migrationsDir: string): Promise<AppliedMigration[]> {
  const historyPath = getHistoryPath(migrationsDir);

  try {
    const content = await fs.readFile(historyPath, 'utf-8');
    const lines = content.trim().split('\n').filter(line => line.trim() && !line.startsWith('#'));

    return lines.map(line => {
      const parts = line.trim().split(/\s+/);
      if (parts.length < 2) {
        throw new Error(`Invalid history line: ${line}`);
      }
      const [id, appliedAt] = parts;
      return { id, appliedAt };
    });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

/**
 * Append a migration entry to the history file.
 */
export async function appendHistory(
  migrationsDir: string,
  id: string,
  appliedAt: string
): Promise<void> {
  const historyPath = getHistoryPath(migrationsDir);

  // Ensure the migrations directory exists
  await fs.mkdir(migrationsDir, { recursive: true });

  const entry = `${id} ${appliedAt}\n`;
  await fs.appendFile(historyPath, entry, 'utf-8');
}

/**
 * Get the current migration state by comparing available migrations against the history.
 */
export async function getState(
  migrationsDir: string,
  availableMigrations: LoadedMigration[]
): Promise<MigrationState> {
  const applied = await readHistory(migrationsDir);
  const appliedIds = new Set(applied.map(m => m.id));

  const pending = availableMigrations.filter(m => !appliedIds.has(m.id));

  return {
    applied,
    pending,
    available: availableMigrations,
  };
}
