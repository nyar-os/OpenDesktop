#!/usr/bin/env node
/** Copy an existing release cdylib into the platform package without building. */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { findCdylib } from './find-cdylib.ts';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const artifact = findCdylib(path.join(root, 'target', 'release'), 'open_desktop_napi');
if (!artifact) {
    console.error('native artifact not found under target/release');
    process.exit(1);
}

const destDir = path.join(root, 'packages', 'open-desktop-win32-x64');
fs.mkdirSync(destDir, { recursive: true });
const dest = path.join(destDir, 'open-desktop.win32-x64.node');
fs.copyFileSync(artifact, dest);
console.log(`staged ${artifact} -> ${dest}`);
