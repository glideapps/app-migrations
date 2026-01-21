import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'fs/promises';
import path from 'path';
import os from 'os';
import { readHistory, appendHistory, getState, getHistoryPath } from './state.js';
import type { LoadedMigration } from './types.js';

describe('state', () => {
  let tempDir: string;
  let migrationsDir: string;

  beforeEach(async () => {
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'app-migrations-test-'));
    migrationsDir = path.join(tempDir, 'migrations');
    await fs.mkdir(migrationsDir, { recursive: true });
  });

  afterEach(async () => {
    await fs.rm(tempDir, { recursive: true, force: true });
  });

  describe('getHistoryPath', () => {
    it('returns the correct path', () => {
      const result = getHistoryPath('/project/migrations');
      expect(result).toBe('/project/migrations/.history');
    });
  });

  describe('readHistory', () => {
    it('returns empty array when history file does not exist', async () => {
      const result = await readHistory(migrationsDir);
      expect(result).toEqual([]);
    });

    it('parses history file correctly', async () => {
      const historyContent = `001-initial 2024-01-15T09:00:00.000Z
002-add-feature 2024-01-16T14:30:00.000Z`;
      await fs.writeFile(path.join(migrationsDir, '.history'), historyContent);

      const result = await readHistory(migrationsDir);

      expect(result).toEqual([
        { id: '001-initial', appliedAt: '2024-01-15T09:00:00.000Z' },
        { id: '002-add-feature', appliedAt: '2024-01-16T14:30:00.000Z' },
      ]);
    });

    it('ignores comment lines', async () => {
      const historyContent = `# This is a comment
001-initial 2024-01-15T09:00:00.000Z`;
      await fs.writeFile(path.join(migrationsDir, '.history'), historyContent);

      const result = await readHistory(migrationsDir);

      expect(result).toEqual([
        { id: '001-initial', appliedAt: '2024-01-15T09:00:00.000Z' },
      ]);
    });

    it('ignores empty lines', async () => {
      const historyContent = `001-initial 2024-01-15T09:00:00.000Z

002-add-feature 2024-01-16T14:30:00.000Z`;
      await fs.writeFile(path.join(migrationsDir, '.history'), historyContent);

      const result = await readHistory(migrationsDir);

      expect(result).toHaveLength(2);
    });
  });

  describe('appendHistory', () => {
    it('creates history file if it does not exist', async () => {
      await appendHistory(migrationsDir, '001-initial', '2024-01-15T09:00:00.000Z');

      const content = await fs.readFile(path.join(migrationsDir, '.history'), 'utf-8');
      expect(content).toBe('001-initial 2024-01-15T09:00:00.000Z\n');
    });

    it('appends to existing history file', async () => {
      await fs.writeFile(path.join(migrationsDir, '.history'), '001-initial 2024-01-15T09:00:00.000Z\n');

      await appendHistory(migrationsDir, '002-add-feature', '2024-01-16T14:30:00.000Z');

      const content = await fs.readFile(path.join(migrationsDir, '.history'), 'utf-8');
      expect(content).toBe('001-initial 2024-01-15T09:00:00.000Z\n002-add-feature 2024-01-16T14:30:00.000Z\n');
    });
  });

  describe('getState', () => {
    it('returns all migrations as pending when history is empty', async () => {
      const mockMigrations: LoadedMigration[] = [
        { id: '001-initial', prefix: 1, filePath: '/path/to/001-initial.ts', module: { up: async () => {} } },
        { id: '002-add-feature', prefix: 2, filePath: '/path/to/002-add-feature.ts', module: { up: async () => {} } },
      ];

      const state = await getState(migrationsDir, mockMigrations);

      expect(state.applied).toEqual([]);
      expect(state.pending).toEqual(mockMigrations);
      expect(state.available).toEqual(mockMigrations);
    });

    it('correctly separates applied and pending migrations', async () => {
      await fs.writeFile(path.join(migrationsDir, '.history'), '001-initial 2024-01-15T09:00:00.000Z\n');

      const mockMigrations: LoadedMigration[] = [
        { id: '001-initial', prefix: 1, filePath: '/path/to/001-initial.ts', module: { up: async () => {} } },
        { id: '002-add-feature', prefix: 2, filePath: '/path/to/002-add-feature.ts', module: { up: async () => {} } },
      ];

      const state = await getState(migrationsDir, mockMigrations);

      expect(state.applied).toEqual([
        { id: '001-initial', appliedAt: '2024-01-15T09:00:00.000Z' },
      ]);
      expect(state.pending).toEqual([mockMigrations[1]]);
    });
  });
});
