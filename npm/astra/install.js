'use strict';

const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const root = path.join(__dirname, '..', '..', '..');
const targetDir = path.join(root, 'target', 'release');

const EXE = process.platform === 'win32' ? 'astra.exe' : 'astra';
const built = path.join(targetDir, EXE);
const localDest = path.join(__dirname, '..', EXE);

function platformSuffix() {
  switch (process.platform) {
    case 'win32':
      return 'win32-x64';
    case 'darwin':
      return 'darwin-arm64';
    case 'linux':
      return 'linux-x64';
    default:
      return null;
  }
}

function log(msg) {
  process.stdout.write(`[zarathustra/cli] ${msg}\n`);
}

function installFrom(src) {
  fs.mkdirSync(path.dirname(localDest), { recursive: true });
  fs.copyFileSync(src, localDest);
  log(`copied ${src}`);

  const suffix = platformSuffix();
  if (suffix) {
    const pkgBinDir = path.join(__dirname, '..', '..', suffix, 'bin');
    fs.mkdirSync(pkgBinDir, { recursive: true });
    fs.copyFileSync(src, path.join(pkgBinDir, EXE));
    log(`copied ${src} -> npm/${suffix}/bin/${EXE}`);
  }
}

if (fs.existsSync(built)) {
  installFrom(built);
} else {
  log(`native binary not found at ${built}; building from source with cargo...`);
  const cargo = spawnSync('cargo', ['build', '--release', '--bin', 'astra'], {
    cwd: root,
    stdio: 'inherit',
  });
  if (cargo.status !== 0) {
    console.error('[zarathustra/cli] cargo build failed. Install a Rust toolchain or provide a prebuilt binary.');
    process.exit(1);
  }
  installFrom(built);
  log('build complete');
}
