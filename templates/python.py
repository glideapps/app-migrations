#!/usr/bin/env python3
# Description: {{DESCRIPTION}}

import json
import os
import shutil
from pathlib import Path

project_root = Path(os.environ['MIGRATE_PROJECT_ROOT'])
migrations_dir = Path(os.environ['MIGRATE_MIGRATIONS_DIR'])
migration_id = os.environ['MIGRATE_ID']
dry_run = os.environ.get('MIGRATE_DRY_RUN', 'false') == 'true'

print(f'Running migration: {migration_id}')

# Example operations (remove or modify as needed):

# 1. Copy file from migration sub-dir to target location
# source_file = migrations_dir / migration_id / 'config.example.json'
# target_file = project_root / 'config' / 'config.json'
# target_file.parent.mkdir(parents=True, exist_ok=True)
# shutil.copy2(source_file, target_file)

# 2. Update a JSON file: remove one element and set another value
# config_path = project_root / 'config.json'
# with open(config_path) as f:
#     config = json.load(f)
# config.pop('oldField', None)
# config.setdefault('settings', {})['newValue'] = 'updated'
# with open(config_path, 'w') as f:
#     json.dump(config, f, indent=2)

# 3. Delete one directory and replace it with another
# old_dir = project_root / 'old-directory'
# new_dir_source = migrations_dir / migration_id / 'new-directory'
# new_dir_target = project_root / 'new-directory'
# shutil.rmtree(old_dir, ignore_errors=True)
# shutil.copytree(new_dir_source, new_dir_target)
