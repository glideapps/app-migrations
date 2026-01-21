import fs from 'fs/promises';
import type { ProjectDirectory } from '../../src/types.js';

export const description = 'Add ESLint configuration';

export async function up(project: ProjectDirectory) {
  // Create .eslintrc.json
  const eslintConfig = {
    extends: ['eslint:recommended'],
    env: {
      node: true,
      es2022: true,
    },
    parserOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
    },
    rules: {
      'no-unused-vars': 'warn',
    },
  };

  await fs.writeFile(
    project.resolve('.eslintrc.json'),
    JSON.stringify(eslintConfig, null, 2)
  );

  // Update package.json to add ESLint
  const pkgPath = project.resolve('package.json');
  const pkg = JSON.parse(await fs.readFile(pkgPath, 'utf-8'));

  pkg.devDependencies = {
    ...pkg.devDependencies,
    eslint: '^8.56.0',
  };

  pkg.scripts = {
    ...pkg.scripts,
    lint: 'eslint src',
  };

  await fs.writeFile(pkgPath, JSON.stringify(pkg, null, 2));
}
