import path from 'path';

/**
 * Represents the project directory that migrations operate on.
 * Provides a safe interface for resolving paths within the project.
 */
export interface ProjectDirectory {
  /** Absolute path to the project root */
  readonly path: string;

  /** Resolve a path relative to the project root */
  resolve(...segments: string[]): string;
}

/**
 * Creates a ProjectDirectory instance for the given root path.
 */
export function createProjectDirectory(rootPath: string): ProjectDirectory {
  const absolutePath = path.resolve(rootPath);
  return {
    path: absolutePath,
    resolve(...segments: string[]): string {
      return path.join(absolutePath, ...segments);
    },
  };
}

/**
 * The contract for a migration module.
 * Each migration must export an `up` function that transforms the project.
 */
export interface Migration {
  /** Optional human-readable description of what this migration does */
  description?: string;

  /** The migration function that transforms the project */
  up: (project: ProjectDirectory) => Promise<void>;
}

/**
 * Represents a migration that has been loaded from disk.
 */
export interface LoadedMigration {
  /** The migration ID (filename without extension, e.g., "001-initial-setup") */
  id: string;

  /** The numeric prefix extracted from the filename */
  prefix: number;

  /** Absolute path to the migration file */
  filePath: string;

  /** The loaded migration module */
  module: Migration;
}

/**
 * Represents a migration that has been applied.
 */
export interface AppliedMigration {
  /** The migration ID */
  id: string;

  /** ISO timestamp when the migration was applied */
  appliedAt: string;
}

/**
 * The current state of migrations for a project.
 */
export interface MigrationState {
  /** Migrations that have already been applied */
  applied: AppliedMigration[];

  /** Migrations that are pending (not yet applied) */
  pending: LoadedMigration[];

  /** All available migrations */
  available: LoadedMigration[];
}

/**
 * Result of applying a single migration.
 */
export interface MigrationResult {
  /** The migration ID */
  id: string;

  /** Whether the migration was successful */
  success: boolean;

  /** ISO timestamp when the migration was applied */
  appliedAt: string;

  /** Error message if the migration failed */
  error?: string;

  /** Whether this was a dry run (no changes made) */
  dryRun: boolean;
}

/**
 * Options for the migrate function.
 */
export interface MigrateOptions {
  /** If true, don't actually apply migrations, just report what would happen */
  dryRun?: boolean;

  /** Path to the migrations directory (default: "migrations" relative to projectRoot) */
  migrationsDir?: string;
}

/**
 * Options for creating a new migration.
 */
export interface CreateMigrationOptions {
  /** Description to include in the migration file */
  description?: string;
}
