#!/usr/bin/env node
import { program } from 'commander';
import path from 'path';
import { getMigrationState, migrate, createMigration, getDefaultMigrationsDir } from './engine.js';

// ANSI color codes
const colors = {
  green: (text: string) => `\x1b[32m${text}\x1b[0m`,
  yellow: (text: string) => `\x1b[33m${text}\x1b[0m`,
  red: (text: string) => `\x1b[31m${text}\x1b[0m`,
  gray: (text: string) => `\x1b[90m${text}\x1b[0m`,
  bold: (text: string) => `\x1b[1m${text}\x1b[0m`,
};

program
  .name('app-migrate')
  .description('File system migration tool')
  .version('1.0.0');

/**
 * Resolve the project root and migrations directory from CLI options.
 */
function resolvePaths(options: { root?: string; migrations?: string }): { projectRoot: string; migrationsDir: string } {
  const projectRoot = options.root
    ? (path.isAbsolute(options.root) ? options.root : path.join(process.cwd(), options.root))
    : process.cwd();

  const migrationsDir = options.migrations
    ? (path.isAbsolute(options.migrations) ? options.migrations : path.join(projectRoot, options.migrations))
    : path.join(projectRoot, 'migrations');

  return { projectRoot, migrationsDir };
}

program
  .command('status')
  .description('Show applied and pending migrations')
  .option('-r, --root <path>', 'Project root directory (default: current directory)')
  .option('-m, --migrations <path>', 'Migrations directory (default: migrations)')
  .action(async (options) => {
    const { projectRoot, migrationsDir } = resolvePaths(options);

    try {
      const state = await getMigrationState(projectRoot, { migrationsDir });

      console.log(colors.bold('\nMigration Status\n'));

      if (state.available.length === 0) {
        console.log(colors.gray('No migrations found in ' + migrationsDir));
        return;
      }

      // Show applied migrations
      if (state.applied.length > 0) {
        console.log(colors.bold('Applied:'));
        for (const applied of state.applied) {
          const migration = state.available.find(m => m.id === applied.id);
          const description = migration?.module.description ?? '';
          console.log(`  ${colors.green('✓')} ${applied.id} ${colors.gray(description)}`);
          console.log(`    ${colors.gray(`Applied: ${applied.appliedAt}`)}`);
        }
        console.log();
      }

      // Show pending migrations
      if (state.pending.length > 0) {
        console.log(colors.bold('Pending:'));
        for (const pending of state.pending) {
          const description = pending.module.description ?? '';
          console.log(`  ${colors.yellow('○')} ${pending.id} ${colors.gray(description)}`);
        }
        console.log();
      }

      // Summary
      const summary = `${state.applied.length} applied, ${state.pending.length} pending`;
      console.log(colors.gray(summary));
    } catch (error) {
      console.error(colors.red(`Error: ${error instanceof Error ? error.message : error}`));
      process.exit(1);
    }
  });

program
  .command('up')
  .description('Apply all pending migrations')
  .option('-r, --root <path>', 'Project root directory (default: current directory)')
  .option('-m, --migrations <path>', 'Migrations directory (default: migrations)')
  .option('--dry-run', 'Preview changes without applying', false)
  .action(async (options) => {
    const { projectRoot, migrationsDir } = resolvePaths(options);

    try {
      if (options.dryRun) {
        console.log(colors.bold('\nDry Run - No changes will be made\n'));
      } else {
        console.log(colors.bold('\nApplying Migrations\n'));
      }

      const results = await migrate(projectRoot, {
        migrationsDir,
        dryRun: options.dryRun,
      });

      if (results.length === 0) {
        console.log(colors.green('✓ Already up to date'));
        return;
      }

      for (const result of results) {
        if (result.success) {
          const prefix = options.dryRun ? colors.yellow('○') : colors.green('✓');
          const action = options.dryRun ? 'Would apply' : 'Applied';
          console.log(`  ${prefix} ${action}: ${result.id}`);
        } else {
          console.log(`  ${colors.red('✗')} Failed: ${result.id}`);
          console.log(`    ${colors.red(result.error ?? 'Unknown error')}`);
        }
      }

      console.log();

      const successful = results.filter(r => r.success).length;
      const failed = results.filter(r => !r.success).length;

      if (options.dryRun) {
        console.log(colors.gray(`${successful} migration(s) would be applied`));
      } else if (failed > 0) {
        console.log(colors.red(`${successful} applied, ${failed} failed`));
        process.exit(1);
      } else {
        console.log(colors.green(`${successful} migration(s) applied successfully`));
      }
    } catch (error) {
      console.error(colors.red(`Error: ${error instanceof Error ? error.message : error}`));
      process.exit(1);
    }
  });

program
  .command('create <name>')
  .description('Create a new migration file')
  .option('-r, --root <path>', 'Project root directory (default: current directory)')
  .option('-m, --migrations <path>', 'Migrations directory (default: migrations)')
  .option('-d, --description <text>', 'Migration description')
  .action(async (name, options) => {
    const { projectRoot, migrationsDir } = resolvePaths(options);

    try {
      const filePath = await createMigration(migrationsDir, name, {
        description: options.description,
      });

      const relativePath = path.relative(projectRoot, filePath);
      console.log(colors.green(`\n✓ Created migration: ${relativePath}\n`));
    } catch (error) {
      console.error(colors.red(`Error: ${error instanceof Error ? error.message : error}`));
      process.exit(1);
    }
  });

program.parse();
