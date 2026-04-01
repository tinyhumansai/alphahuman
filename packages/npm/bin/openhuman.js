#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');
const path = require('path');

const isWin = process.platform === 'win32';
const binName = isWin ? 'openhuman-bin.exe' : 'openhuman-bin';
const binPath = path.join(__dirname, binName);

const result = spawnSync(binPath, process.argv.slice(2), {
  stdio: 'inherit',
  windowsHide: false,
});

if (result.error) {
  if (result.error.code === 'ENOENT') {
    process.stderr.write(
      'openhuman binary not found. Try reinstalling: npm install -g openhuman\n'
    );
  } else {
    process.stderr.write(`openhuman: ${result.error.message}\n`);
  }
  process.exit(1);
}

process.exit(result.status ?? 0);
