#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');
const os = require('os');

const binName = os.platform() === 'win32' ? 'chronx.exe' : 'chronx';
const binPath = path.join(__dirname, 'bin', binName);

const args = process.argv.slice(2);
const result = spawnSync(binPath, args, { stdio: 'inherit' });
process.exit(result.status || 0);
