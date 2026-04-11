#!/usr/bin/env bun

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

type SupportedTarget = {
  id: string;
  os: NodeJS.Platform;
  cpu: NodeJS.Architecture;
  binaryName: string;
  packageName: string;
};

const workspaceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
);

const npmRoot = path.join(workspaceRoot, 'npm');
const uzumakiPkgPath = path.join(
  workspaceRoot,
  'crates',
  'uzumaki',
  'package.json',
);
const targetRoot = path.join(workspaceRoot, 'target');

const supportedTargets: SupportedTarget[] = [
  {
    id: 'win32-x64',
    os: 'win32',
    cpu: 'x64',
    binaryName: 'uzumaki.exe',
    packageName: '@uzumaki-apps/win32-x64',
  },
  {
    id: 'win32-arm64',
    os: 'win32',
    cpu: 'arm64',
    binaryName: 'uzumaki.exe',
    packageName: '@uzumaki-apps/win32-arm64',
  },
  {
    id: 'linux-x64',
    os: 'linux',
    cpu: 'x64',
    binaryName: 'uzumaki',
    packageName: '@uzumaki-apps/linux-x64',
  },
  {
    id: 'linux-arm64',
    os: 'linux',
    cpu: 'arm64',
    binaryName: 'uzumaki',
    packageName: '@uzumaki-apps/linux-arm64',
  },
  {
    id: 'darwin-x64',
    os: 'darwin',
    cpu: 'x64',
    binaryName: 'uzumaki',
    packageName: '@uzumaki-apps/darwin-x64',
  },
  {
    id: 'darwin-arm64',
    os: 'darwin',
    cpu: 'arm64',
    binaryName: 'uzumaki',
    packageName: '@uzumaki-apps/darwin-arm64',
  },
];

function parseArgValue(flag: string) {
  const argv = process.argv.slice(2);
  const index = argv.indexOf(flag);
  if (index === -1) return;
  return argv[index + 1];
}

function hasFlag(flag: string) {
  return process.argv.slice(2).includes(flag);
}

function getCurrentTargetId() {
  const match = supportedTargets.find(
    (target) => target.os === process.platform && target.cpu === process.arch,
  );
  if (!match) {
    throw new Error(
      `unsupported platform/arch combination: ${process.platform}-${process.arch}`,
    );
  }
  return match.id;
}

function getTarget(targetId: string) {
  const target = supportedTargets.find((entry) => entry.id === targetId);
  if (!target) {
    const supported = supportedTargets.map((entry) => entry.id).join(', ');
    throw new Error(
      `unknown target "${targetId}". supported targets: ${supported}`,
    );
  }
  return target;
}

function ensureDir(dir: string) {
  fs.mkdirSync(dir, { recursive: true });
}

function writeFile(filePath: string, contents: string) {
  fs.writeFileSync(filePath, contents, 'utf8');
}

function writePackageJson(
  target: SupportedTarget,
  version: string,
  outDir: string,
) {
  const pkg = {
    name: target.packageName,
    version,
    type: 'module',
    private: false,
    os: [target.os],
    cpu: [target.cpu],
    files: ['index.js', target.binaryName],
    exports: {
      '.': './index.js',
    },
    sideEffects: false,
  };

  writeFile(
    path.join(outDir, 'package.json'),
    `${JSON.stringify(pkg, null, 2)}\n`,
  );
}

function writeIndexJs(target: SupportedTarget, outDir: string) {
  const contents = `import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dirname = path.dirname(fileURLToPath(import.meta.url));

const binaryName = ${JSON.stringify(target.binaryName)};
const binaryPath = path.join(dirname, binaryName);

export default binaryPath;
`;

  writeFile(path.join(outDir, 'index.js'), contents);
}

function resolveVersion() {
  const explicitVersion = parseArgValue('--version');
  if (explicitVersion) return explicitVersion;

  const uzumakiPkg = JSON.parse(fs.readFileSync(uzumakiPkgPath, 'utf8')) as {
    version?: string;
  };

  return uzumakiPkg.version ?? '0.1.0';
}

function resolveBinarySource(target: SupportedTarget) {
  const explicitBinary = parseArgValue('--binary');
  if (explicitBinary) {
    return path.resolve(workspaceRoot, explicitBinary);
  }

  const profile = parseArgValue('--profile') ?? 'release';
  return path.join(targetRoot, profile, target.binaryName);
}

function copyBinary(sourcePath: string, targetPath: string) {
  fs.copyFileSync(sourcePath, targetPath);
}

function makeExecutableIfNeeded(target: SupportedTarget, binaryPath: string) {
  if (target.os !== 'win32') {
    fs.chmodSync(binaryPath, 0o755);
  }
}

function cleanDirContents(dir: string) {
  if (!fs.existsSync(dir)) return;
  for (const entry of fs.readdirSync(dir)) {
    fs.rmSync(path.join(dir, entry), { recursive: true, force: true });
  }
}

function generateTarget(target: SupportedTarget, version: string) {
  const outDir = path.join(npmRoot, target.id);
  const sourceBinary = resolveBinarySource(target);

  if (!fs.existsSync(sourceBinary)) {
    throw new Error(
      `binary not found for ${target.id}: ${sourceBinary}\npass --binary <path> to override`,
    );
  }

  ensureDir(outDir);
  cleanDirContents(outDir);

  writePackageJson(target, version, outDir);
  writeIndexJs(target, outDir);

  const outBinary = path.join(outDir, target.binaryName);
  copyBinary(sourceBinary, outBinary);
  makeExecutableIfNeeded(target, outBinary);

  console.log(`generated ${target.packageName} -> ${outDir}`);
}

function main() {
  ensureDir(npmRoot);

  const version = resolveVersion();
  const targetArg = parseArgValue('--platform') ?? parseArgValue('--target');

  if (hasFlag('--all')) {
    for (const target of supportedTargets) {
      try {
        generateTarget(target, version);
      } catch (error) {
        console.warn(String(error));
      }
    }
    return;
  }

  const target = getTarget(targetArg ?? getCurrentTargetId());
  generateTarget(target, version);
}

main();
