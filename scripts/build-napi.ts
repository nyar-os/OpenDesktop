#!/usr/bin/env node
/** Build the Node-API cdylib and stage it into the platform package. */
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { findCdylib } from './find-cdylib.ts';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const crateName = 'open-desktop-napi';
const cdylibName = 'open_desktop_napi';

const build = spawnSync('cargo', ['build', '-p', crateName, '--release'], {
    cwd: root,
    stdio: 'inherit',
    shell: false,
});
if ((build.status ?? 1) !== 0) {
    process.exit(build.status ?? 1);
}

const artifact = findCdylib(path.join(root, 'target', 'release'), cdylibName);
if (!artifact) {
    console.error(`native artifact not found for ${crateName} under target/release`);
    process.exit(1);
}

const destDir = path.join(root, 'packages', 'open-desktop-win32-x64');
fs.mkdirSync(destDir, { recursive: true });
const dest = path.join(destDir, 'open-desktop.win32-x64.node');
fs.copyFileSync(artifact, dest);
console.log(`staged ${artifact} -> ${dest}`);
