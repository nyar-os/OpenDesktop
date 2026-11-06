/** Locate a release cdylib inside the Cargo target directory. */
import fs from 'node:fs';
import path from 'node:path';

/**
 * @param releaseDir Cargo `target/release` directory.
 * @param cdylibName Library name with underscores, for example `open_desktop_napi`.
 */
export function findCdylib(releaseDir: string, cdylibName: string): string | null {
    const candidates = [
        `${cdylibName}.dll`,
        `${cdylibName}.node`,
        `lib${cdylibName}.so`,
        `lib${cdylibName}.dylib`,
    ].map((name) => path.join(releaseDir, name));

    return candidates.find((candidate) => fs.existsSync(candidate)) ?? null;
}
