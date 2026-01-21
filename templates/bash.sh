#!/usr/bin/env bash
set -euo pipefail
# Description: {{DESCRIPTION}}

cd "$MIGRATE_PROJECT_ROOT"

echo "Running migration: $MIGRATE_ID"

# Example operations (remove or modify as needed):

# 1. Copy file from migration sub-dir to target location
# cp "$MIGRATE_MIGRATIONS_DIR/$MIGRATE_ID/config.example.json" ./config/config.json

# 2. Update a JSON file: remove one element and set another value
# jq 'del(.oldField) | .settings.newValue = "updated"' config.json > config.json.tmp
# mv config.json.tmp config.json

# 3. Delete one directory and replace it with another
# rm -rf ./old-directory
# cp -r "$MIGRATE_MIGRATIONS_DIR/$MIGRATE_ID/new-directory" ./new-directory
