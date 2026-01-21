#!/usr/bin/env node
// Description: {{DESCRIPTION}}

const fs = require('fs').promises;
const path = require('path');

const projectRoot = process.env.MIGRATE_PROJECT_ROOT;
const migrationsDir = process.env.MIGRATE_MIGRATIONS_DIR;
const migrationId = process.env.MIGRATE_ID;
const dryRun = process.env.MIGRATE_DRY_RUN === 'true';

async function main() {
    console.log(`Running migration: ${migrationId}`);

    // Example operations (remove or modify as needed):

    // 1. Copy file from migration sub-dir to target location
    // const sourceFile = path.join(migrationsDir, migrationId, 'config.example.json');
    // const targetFile = path.join(projectRoot, 'config', 'config.json');
    // await fs.mkdir(path.dirname(targetFile), { recursive: true });
    // await fs.copyFile(sourceFile, targetFile);

    // 2. Update a JSON file: remove one element and set another value
    // const configPath = path.join(projectRoot, 'config.json');
    // const config = JSON.parse(await fs.readFile(configPath, 'utf-8'));
    // delete config.oldField;
    // config.settings = config.settings || {};
    // config.settings.newValue = 'updated';
    // await fs.writeFile(configPath, JSON.stringify(config, null, 2));

    // 3. Delete one directory and replace it with another
    // const oldDir = path.join(projectRoot, 'old-directory');
    // const newDirSource = path.join(migrationsDir, migrationId, 'new-directory');
    // const newDirTarget = path.join(projectRoot, 'new-directory');
    // await fs.rm(oldDir, { recursive: true, force: true });
    // await copyDir(newDirSource, newDirTarget);
}

// Helper: recursively copy a directory
async function copyDir(src, dest) {
    await fs.mkdir(dest, { recursive: true });
    const entries = await fs.readdir(src, { withFileTypes: true });
    for (const entry of entries) {
        const srcPath = path.join(src, entry.name);
        const destPath = path.join(dest, entry.name);
        if (entry.isDirectory()) {
            await copyDir(srcPath, destPath);
        } else {
            await fs.copyFile(srcPath, destPath);
        }
    }
}

main();
