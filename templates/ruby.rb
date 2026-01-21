#!/usr/bin/env ruby
# Description: {{DESCRIPTION}}

require 'json'
require 'fileutils'

project_root = ENV['MIGRATE_PROJECT_ROOT']
migrations_dir = ENV['MIGRATE_MIGRATIONS_DIR']
migration_id = ENV['MIGRATE_ID']
dry_run = ENV['MIGRATE_DRY_RUN'] == 'true'

puts "Running migration: #{migration_id}"

# Example operations (remove or modify as needed):

# 1. Copy file from migration sub-dir to target location
# source_file = File.join(migrations_dir, migration_id, 'config.example.json')
# target_file = File.join(project_root, 'config', 'config.json')
# FileUtils.mkdir_p(File.dirname(target_file))
# FileUtils.cp(source_file, target_file)

# 2. Update a JSON file: remove one element and set another value
# config_path = File.join(project_root, 'config.json')
# config = JSON.parse(File.read(config_path))
# config.delete('oldField')
# config['settings'] ||= {}
# config['settings']['newValue'] = 'updated'
# File.write(config_path, JSON.pretty_generate(config))

# 3. Delete one directory and replace it with another
# old_dir = File.join(project_root, 'old-directory')
# new_dir_source = File.join(migrations_dir, migration_id, 'new-directory')
# new_dir_target = File.join(project_root, 'new-directory')
# FileUtils.rm_rf(old_dir)
# FileUtils.cp_r(new_dir_source, new_dir_target)
