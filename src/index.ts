// Public API exports for app-migrations

// Types
export type {
  ProjectDirectory,
  Migration,
  LoadedMigration,
  AppliedMigration,
  MigrationState,
  MigrationResult,
  MigrateOptions,
  CreateMigrationOptions,
} from './types.js';

// Type utilities
export { createProjectDirectory } from './types.js';

// Core functions
export { getMigrationState, migrate, createMigration, getDefaultMigrationsDir } from './engine.js';

// State functions (for advanced use cases)
export { readHistory, appendHistory, getState, getHistoryPath } from './state.js';

// Loader functions (for advanced use cases)
export { loadMigrations, loadMigrationFile, discoverMigrations, listMigrationFiles } from './loader.js';
