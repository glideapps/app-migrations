#!/usr/bin/env -S npx tsx
// Description: {{DESCRIPTION}}

import * as fs from 'fs/promises';
import * as path from 'path';

const projectRoot = process.env.MIGRATE_PROJECT_ROOT!;
const migrationId = process.env.MIGRATE_ID!;
const dryRun = process.env.MIGRATE_DRY_RUN === 'true';

console.log(`Running migration: ${migrationId}`);

// Your migration code here
