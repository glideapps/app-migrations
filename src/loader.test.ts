import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'fs/promises';
import path from 'path';
import os from 'os';
import { extractPrefix, extractId, discoverMigrations, loadMigrationFile, loadMigrations } from './loader.js';

describe('loader', () => {
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

  describe('extractPrefix', () => {
    it('extracts numeric prefix from filename', () => {
      expect(extractPrefix('001-initial-setup.ts')).toBe(1);
      expect(extractPrefix('042-add-feature.ts')).toBe(42);
      expect(extractPrefix('100-big-change.ts')).toBe(100);
    });

    it('returns null for invalid filenames', () => {
      expect(extractPrefix('initial-setup.ts')).toBeNull();
      expect(extractPrefix('001.ts')).toBeNull();
      expect(extractPrefix('001-setup.js')).toBeNull();
      expect(extractPrefix('abc-setup.ts')).toBeNull();
    });
  });

  describe('extractId', () => {
    it('extracts id from filename', () => {
      expect(extractId('001-initial-setup.ts')).toBe('001-initial-setup');
      expect(extractId('042-add-feature.ts')).toBe('042-add-feature');
    });
  });

  describe('discoverMigrations', () => {
    it('returns empty array when no migrations exist', async () => {
      const result = await discoverMigrations(migrationsDir);
      expect(result).toEqual([]);
    });

    it('discovers migration files sorted by prefix', async () => {
      await fs.writeFile(path.join(migrationsDir, '002-second.ts'), 'export const up = () => {}');
      await fs.writeFile(path.join(migrationsDir, '001-first.ts'), 'export const up = () => {}');
      await fs.writeFile(path.join(migrationsDir, '003-third.ts'), 'export const up = () => {}');

      const result = await discoverMigrations(migrationsDir);

      expect(result).toHaveLength(3);
      expect(path.basename(result[0])).toBe('001-first.ts');
      expect(path.basename(result[1])).toBe('002-second.ts');
      expect(path.basename(result[2])).toBe('003-third.ts');
    });

    it('ignores non-matching files', async () => {
      await fs.writeFile(path.join(migrationsDir, '001-valid.ts'), 'export const up = () => {}');
      await fs.writeFile(path.join(migrationsDir, 'invalid.ts'), 'export const up = () => {}');
      await fs.writeFile(path.join(migrationsDir, '001.ts'), 'export const up = () => {}');
      await fs.writeFile(path.join(migrationsDir, 'README.md'), '# Migrations');

      const result = await discoverMigrations(migrationsDir);

      expect(result).toHaveLength(1);
      expect(path.basename(result[0])).toBe('001-valid.ts');
    });
  });

  describe('loadMigrationFile', () => {
    it('loads a valid migration file', async () => {
      const migrationContent = `
export const description = 'Test migration';
export async function up(project) {
  // do nothing
}
`;
      const filePath = path.join(migrationsDir, '001-test.ts');
      await fs.writeFile(filePath, migrationContent);

      const result = await loadMigrationFile(filePath);

      expect(result.id).toBe('001-test');
      expect(result.prefix).toBe(1);
      expect(result.filePath).toBe(filePath);
      expect(result.module.description).toBe('Test migration');
      expect(typeof result.module.up).toBe('function');
    });

    it('throws error for migration without up function', async () => {
      const migrationContent = `
export const description = 'Invalid migration';
`;
      const filePath = path.join(migrationsDir, '001-invalid.ts');
      await fs.writeFile(filePath, migrationContent);

      await expect(loadMigrationFile(filePath)).rejects.toThrow("must export an 'up' function");
    });
  });

  describe('loadMigrations', () => {
    it('loads all migrations in order', async () => {
      const migration1 = `
export const description = 'First';
export async function up() {}
`;
      const migration2 = `
export const description = 'Second';
export async function up() {}
`;
      await fs.writeFile(path.join(migrationsDir, '001-first.ts'), migration1);
      await fs.writeFile(path.join(migrationsDir, '002-second.ts'), migration2);

      const result = await loadMigrations(migrationsDir);

      expect(result).toHaveLength(2);
      expect(result[0].id).toBe('001-first');
      expect(result[0].module.description).toBe('First');
      expect(result[1].id).toBe('002-second');
      expect(result[1].module.description).toBe('Second');
    });
  });
});
