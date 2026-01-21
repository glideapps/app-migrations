import fs from 'fs/promises';
import type { ProjectDirectory } from '../../src/types.js';

export const description = 'Add TypeScript configuration';

export async function up(project: ProjectDirectory) {
  // Create tsconfig.json
  const tsconfig = {
    compilerOptions: {
      target: 'ES2022',
      module: 'NodeNext',
      moduleResolution: 'NodeNext',
      strict: true,
      outDir: './dist',
      rootDir: './src',
    },
    include: ['src/**/*'],
  };

  await fs.writeFile(
    project.resolve('tsconfig.json'),
    JSON.stringify(tsconfig, null, 2)
  );

  // Update package.json to add TypeScript dev dependency
  const pkgPath = project.resolve('package.json');
  const pkg = JSON.parse(await fs.readFile(pkgPath, 'utf-8'));

  pkg.devDependencies = {
    ...pkg.devDependencies,
    typescript: '^5.3.0',
  };

  pkg.scripts = {
    ...pkg.scripts,
    build: 'tsc',
  };

  await fs.writeFile(pkgPath, JSON.stringify(pkg, null, 2));
}
