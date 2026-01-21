import { describe, it, expect, beforeAll, beforeEach, afterEach } from 'vitest';
import { execSync } from 'child_process';
import fs from 'fs/promises';
import path from 'path';
import os from 'os';

const projectRoot = path.resolve(import.meta.dirname, '..');
const cliBinary = path.join(projectRoot, 'dist', 'cli.cjs');

describe('CLI executable', () => {
  beforeAll(async () => {
    // Build the CLI before running tests
    execSync('npm run build', { cwd: projectRoot, stdio: 'pipe' });

    // Verify the CLI was built
    await fs.access(cliBinary);
  });

  describe('binary execution', () => {
    it('is executable and shows help', () => {
      const result = execSync(`node ${cliBinary} --help`, { encoding: 'utf-8' });

      expect(result).toContain('app-migrate');
      expect(result).toContain('File system migration tool');
      expect(result).toContain('status');
      expect(result).toContain('up');
      expect(result).toContain('create');
    });

    it('shows version from package.json', async () => {
      const pkg = JSON.parse(await fs.readFile(path.join(projectRoot, 'package.json'), 'utf-8'));
      const result = execSync(`node ${cliBinary} --version`, { encoding: 'utf-8' });

      expect(result.trim()).toBe(pkg.version);
    });
  });

  describe('status command', () => {
    let tempDir: string;
    let testProjectRoot: string;
    let migrationsDir: string;

    beforeEach(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'app-migrations-cli-test-'));
      testProjectRoot = tempDir;
      migrationsDir = path.join(testProjectRoot, 'migrations');
      await fs.mkdir(migrationsDir, { recursive: true });
    });

    afterEach(async () => {
      await fs.rm(tempDir, { recursive: true, force: true });
    });

    it('shows no migrations when directory is empty', () => {
      const result = execSync(`node ${cliBinary} status -r ${testProjectRoot}`, {
        encoding: 'utf-8',
      });

      expect(result).toContain('Migration Status');
      expect(result).toContain('No migrations found');
    });

    it('shows pending migrations', async () => {
      const migration = `export async function up() {}`;
      await fs.writeFile(path.join(migrationsDir, '001-test-migration.mjs'), migration);

      const result = execSync(`node ${cliBinary} status -r ${testProjectRoot}`, {
        encoding: 'utf-8',
      });

      expect(result).toContain('Pending');
      expect(result).toContain('001-test-migration');
      expect(result).toContain('0 applied, 1 pending');
    });
  });

  describe('up command', () => {
    let tempDir: string;
    let testProjectRoot: string;
    let migrationsDir: string;

    beforeEach(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'app-migrations-cli-test-'));
      testProjectRoot = tempDir;
      migrationsDir = path.join(testProjectRoot, 'migrations');
      await fs.mkdir(migrationsDir, { recursive: true });
    });

    afterEach(async () => {
      await fs.rm(tempDir, { recursive: true, force: true });
    });

    it('applies migrations', async () => {
      const migration = `
import fs from 'fs/promises';

export async function up(project) {
  await fs.writeFile(project.resolve('created-by-cli.txt'), 'CLI test');
}
`;
      await fs.writeFile(path.join(migrationsDir, '001-create-file.mjs'), migration);

      const result = execSync(`node ${cliBinary} up -r ${testProjectRoot}`, {
        encoding: 'utf-8',
      });

      expect(result).toContain('Applying Migrations');
      expect(result).toContain('Applied: 001-create-file');
      expect(result).toContain('1 migration(s) applied successfully');

      // Verify the file was created
      const content = await fs.readFile(path.join(testProjectRoot, 'created-by-cli.txt'), 'utf-8');
      expect(content).toBe('CLI test');
    });

    it('supports dry-run flag', async () => {
      const migration = `
import fs from 'fs/promises';

export async function up(project) {
  await fs.writeFile(project.resolve('should-not-exist.txt'), 'content');
}
`;
      await fs.writeFile(path.join(migrationsDir, '001-test.mjs'), migration);

      const result = execSync(`node ${cliBinary} up --dry-run -r ${testProjectRoot}`, {
        encoding: 'utf-8',
      });

      expect(result).toContain('Dry Run');
      expect(result).toContain('Would apply: 001-test');
      expect(result).toContain('1 migration(s) would be applied');

      // Verify file was not created
      await expect(fs.access(path.join(testProjectRoot, 'should-not-exist.txt'))).rejects.toThrow();
    });

    it('reports already up to date', async () => {
      const result = execSync(`node ${cliBinary} up -r ${testProjectRoot}`, {
        encoding: 'utf-8',
      });

      expect(result).toContain('Already up to date');
    });
  });

  describe('create command', () => {
    let tempDir: string;
    let testProjectRoot: string;
    let migrationsDir: string;

    beforeEach(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'app-migrations-cli-test-'));
      testProjectRoot = tempDir;
      migrationsDir = path.join(testProjectRoot, 'migrations');
      await fs.mkdir(migrationsDir, { recursive: true });
    });

    afterEach(async () => {
      await fs.rm(tempDir, { recursive: true, force: true });
    });

    it('creates a new migration file', async () => {
      const result = execSync(`node ${cliBinary} create "add feature" -r ${testProjectRoot}`, {
        encoding: 'utf-8',
      });

      expect(result).toContain('Created migration');
      expect(result).toContain('001-add-feature.ts');

      // Verify the file exists and has correct content
      const filePath = path.join(migrationsDir, '001-add-feature.ts');
      const content = await fs.readFile(filePath, 'utf-8');
      expect(content).toContain('export async function up');
    });

    it('supports description option', async () => {
      const result = execSync(
        `node ${cliBinary} create "test" -d "My custom description" -r ${testProjectRoot}`,
        { encoding: 'utf-8' }
      );

      expect(result).toContain('Created migration');

      const filePath = path.join(migrationsDir, '001-test.ts');
      const content = await fs.readFile(filePath, 'utf-8');
      expect(content).toContain('My custom description');
    });
  });

  describe('error handling', () => {
    let tempDir: string;
    let testProjectRoot: string;
    let migrationsDir: string;

    beforeEach(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'app-migrations-cli-test-'));
      testProjectRoot = tempDir;
      migrationsDir = path.join(testProjectRoot, 'migrations');
      await fs.mkdir(migrationsDir, { recursive: true });
    });

    afterEach(async () => {
      await fs.rm(tempDir, { recursive: true, force: true });
    });

    it('exits with error code when migration fails', async () => {
      const migration = `
export async function up() {
  throw new Error('Intentional failure');
}
`;
      await fs.writeFile(path.join(migrationsDir, '001-failing.mjs'), migration);

      try {
        execSync(`node ${cliBinary} up -r ${testProjectRoot}`, {
          encoding: 'utf-8',
          stdio: 'pipe',
        });
        expect.fail('Expected command to throw');
      } catch (error: unknown) {
        const execError = error as { status: number; stdout: string };
        expect(execError.status).toBe(1);
        expect(execError.stdout).toContain('Failed: 001-failing');
      }
    });
  });

  describe('custom migrations directory', () => {
    let tempDir: string;
    let testProjectRoot: string;

    beforeEach(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'app-migrations-cli-test-'));
      testProjectRoot = tempDir;
      await fs.mkdir(path.join(testProjectRoot, 'custom-migrations'), { recursive: true });
    });

    afterEach(async () => {
      await fs.rm(tempDir, { recursive: true, force: true });
    });

    it('uses custom migrations directory with -m flag', async () => {
      const migration = `export async function up() {}`;
      await fs.writeFile(
        path.join(testProjectRoot, 'custom-migrations', '001-custom.mjs'),
        migration
      );

      const result = execSync(
        `node ${cliBinary} status -r ${testProjectRoot} -m custom-migrations`,
        { encoding: 'utf-8' }
      );

      expect(result).toContain('001-custom');
      expect(result).toContain('1 pending');
    });
  });
});
