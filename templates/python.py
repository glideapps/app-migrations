#!/usr/bin/env python3
# Description: {{DESCRIPTION}}

import os

project_root = os.environ['MIGRATE_PROJECT_ROOT']
migration_id = os.environ['MIGRATE_ID']
dry_run = os.environ.get('MIGRATE_DRY_RUN', 'false') == 'true'

print(f'Running migration: {migration_id}')

# Your migration code here
