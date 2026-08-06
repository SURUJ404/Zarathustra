#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');
const path = require('path');

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

function exeName() {
  return process.platform === 'win32' ? 'astra.exe' : 'astra';
}

function resolveBinary() {
  const pkgSuffix = platformSuffix();
  let binPath = null;
  if (pkgSuffix) {
    try {
      binPath = require.resolve(`@zarathustra/cli-${pkgSuffix}/bin/${exeName()}`);
    } catch (_) {
      binPath = null;
    }
  }
  if (!binPath) {
    binPath = path.join(__dirname, '..', exeName());
  }
  return binPath;
}

const bin = resolveBinary();
const result = spawnSync(bin, process.argv.slice(2), {
  stdio: 'inherit',
  windowsHide: true,
});

if (result.error) {
  console.error(`astra: failed to run native binary (${bin}): ${result.error.message}`);
  console.error('astra: ensure it was built via `npm run build` in the repo root, or that the platform package is installed.');
  process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
