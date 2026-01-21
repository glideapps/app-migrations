#!/usr/bin/env node
// Description: {{DESCRIPTION}}

const fs = require('fs').promises;
const path = require('path');

const projectRoot = process.env.MIGRATE_PROJECT_ROOT;
const migrationId = process.env.MIGRATE_ID;
const dryRun = process.env.MIGRATE_DRY_RUN === 'true';

console.log(`Running migration: ${migrationId}`);

// Your migration code here
