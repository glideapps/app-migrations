import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'fs/promises';
import path from 'path';
import os from 'os';
import { getMigrationState, migrate, createMigration } from './engine.js';

describe('engine', () => {
  let tempDir: string;
  let projectRoot: string;
  let migrationsDir: string;

  beforeEach(async () => {
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'app-migrations-test-'));
    projectRoot = tempDir;
    migrationsDir = path.join(projectRoot, 'migrations');
    await fs.mkdir(migrationsDir, { recursive: true });

    // Create a basic package.json in the project
    await fs.writeFile(
      path.join(projectRoot, 'package.json'),
      JSON.stringify({ name: 'test-project', version: '1.0.0' }, null, 2)
    );
  });

  afterEach(async () => {
    await fs.rm(tempDir, { recursive: true, force: true });
  });

  describe('getMigrationState', () => {
    it('returns empty state when no migrations exist', async () => {
      const state = await getMigrationState(projectRoot);

      expect(state.applied).toEqual([]);
      expect(state.pending).toEqual([]);
      expect(state.available).toEqual([]);
    });

    it('returns pending migrations when none are applied', async () => {
      const migration = `
export async function up() {}
`;
      await fs.writeFile(path.join(migrationsDir, '001-test.ts'), migration);

      const state = await getMigrationState(projectRoot);

      expect(state.applied).toEqual([]);
      expect(state.pending).toHaveLength(1);
      expect(state.pending[0].id).toBe('001-test');
    });
  });

  describe('migrate', () => {
    it('applies pending migrations', async () => {
      const migration = `
import fs from 'fs/promises';

export async function up(project) {
  await fs.writeFile(project.resolve('test-file.txt'), 'Hello from migration');
}
`;
      await fs.writeFile(path.join(migrationsDir, '001-create-file.ts'), migration);

      const results = await migrate(projectRoot);

      expect(results).toHaveLength(1);
      expect(results[0].success).toBe(true);
      expect(results[0].id).toBe('001-create-file');
      expect(results[0].dryRun).toBe(false);

      // Verify the file was created
      const content = await fs.readFile(path.join(projectRoot, 'test-file.txt'), 'utf-8');
      expect(content).toBe('Hello from migration');

      // Verify history was updated
      const history = await fs.readFile(path.join(migrationsDir, '.history'), 'utf-8');
      expect(history).toContain('001-create-file');
    });

    it('applies migrations in order', async () => {
      const migration1 = `
import fs from 'fs/promises';

export async function up(project) {
  await fs.writeFile(project.resolve('order.txt'), '1');
}
`;
      const migration2 = `
import fs from 'fs/promises';

export async function up(project) {
  const current = await fs.readFile(project.resolve('order.txt'), 'utf-8');
  await fs.writeFile(project.resolve('order.txt'), current + '2');
}
`;
      await fs.writeFile(path.join(migrationsDir, '001-first.ts'), migration1);
      await fs.writeFile(path.join(migrationsDir, '002-second.ts'), migration2);

      const results = await migrate(projectRoot);

      expect(results).toHaveLength(2);
      expect(results[0].id).toBe('001-first');
      expect(results[1].id).toBe('002-second');

      const content = await fs.readFile(path.join(projectRoot, 'order.txt'), 'utf-8');
      expect(content).toBe('12');
    });

    it('returns empty array when all migrations are applied', async () => {
      const migration = `
export async function up() {}
`;
      await fs.writeFile(path.join(migrationsDir, '001-test.ts'), migration);

      // Apply first time
      await migrate(projectRoot);

      // Apply second time
      const results = await migrate(projectRoot);

      expect(results).toEqual([]);
    });

    it('supports dry run mode', async () => {
      const migration = `
import fs from 'fs/promises';

export async function up(project) {
  await fs.writeFile(project.resolve('should-not-exist.txt'), 'content');
}
`;
      await fs.writeFile(path.join(migrationsDir, '001-test.ts'), migration);

      const results = await migrate(projectRoot, { dryRun: true });

      expect(results).toHaveLength(1);
      expect(results[0].success).toBe(true);
      expect(results[0].dryRun).toBe(true);

      // Verify file was not created
      await expect(fs.access(path.join(projectRoot, 'should-not-exist.txt'))).rejects.toThrow();

      // Verify history was not updated
      await expect(fs.access(path.join(migrationsDir, '.history'))).rejects.toThrow();
    });

    it('stops on error and reports failure', async () => {
      const migration1 = `
export async function up() {}
`;
      const migration2 = `
export async function up() {
  throw new Error('Migration failed');
}
`;
      const migration3 = `
export async function up() {}
`;
      await fs.writeFile(path.join(migrationsDir, '001-pass.ts'), migration1);
      await fs.writeFile(path.join(migrationsDir, '002-fail.ts'), migration2);
      await fs.writeFile(path.join(migrationsDir, '003-never-run.ts'), migration3);

      const results = await migrate(projectRoot);

      expect(results).toHaveLength(2);
      expect(results[0].success).toBe(true);
      expect(results[1].success).toBe(false);
      expect(results[1].error).toBe('Migration failed');

      // Third migration should not have run
      const state = await getMigrationState(projectRoot);
      expect(state.applied).toHaveLength(1);
      expect(state.pending).toHaveLength(2);
    });
  });

  describe('createMigration', () => {
    it('creates a new migration file', async () => {
      const filePath = await createMigration(migrationsDir, 'add feature');

      expect(path.basename(filePath)).toBe('001-add-feature.ts');

      const content = await fs.readFile(filePath, 'utf-8');
      expect(content).toContain('export async function up');
      expect(content).toContain('ProjectDirectory');
    });

    it('increments prefix based on existing migrations', async () => {
      await fs.writeFile(
        path.join(migrationsDir, '001-existing.ts'),
        'export async function up() {}'
      );
      await fs.writeFile(
        path.join(migrationsDir, '002-another.ts'),
        'export async function up() {}'
      );

      const filePath = await createMigration(migrationsDir, 'new migration');

      expect(path.basename(filePath)).toBe('003-new-migration.ts');
    });

    it('includes custom description', async () => {
      const filePath = await createMigration(migrationsDir, 'test', {
        description: 'Custom description',
      });

      const content = await fs.readFile(filePath, 'utf-8');
      expect(content).toContain("description = 'Custom description'");
    });

    it('sanitizes migration name', async () => {
      const filePath = await createMigration(migrationsDir, 'Add User@Auth!');

      expect(path.basename(filePath)).toBe('001-add-userauth.ts');
    });
  });
});
