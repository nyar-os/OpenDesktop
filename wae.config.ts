/**
 * Build and release configuration for OpenDesktop.
 *
 * Pure literals only: `wae` parses this file statically and never executes it.
 * Build order is fixed: native → ts → ui → skills → staging.
 */
export default {
    native: {
        crate: 'open-desktop-napi',
        cdylib: 'open_desktop_napi',
        targets: [
            {
                platform: 'win32',
                arch: 'x64',
                triple: 'x86_64-pc-windows-msvc',
                package: '@wae/open-desktop-win32-x64',
                nodeFile: 'open-desktop.win32-x64.node',
                dir: 'packages/open-desktop-win32-x64',
            },
        ],
    },
    packages: [
        {
            name: '@wae/open-desktop-win32-x64',
            dir: 'packages/open-desktop-win32-x64',
            os: ['win32'],
            cpu: ['x64'],
            main: 'open-desktop.win32-x64.node',
        },
    ],
    build: {
        ts: { packages: ['@wae/open-desktop'] },
        ui: { package: '@wae/open-desktop-ui' },
        stage: [
            { from: 'packages/open-desktop-ui/dist/static', to: 'packages/open-desktop/dist/gui' },
        ],
        outputs: ['packages/open-desktop/dist/cli.js', 'packages/open-desktop/dist/gui/index.html'],
    },
    skills: {
        packages: [
            {
                dir: 'packages/open-desktop-skills/skills',
                packageJson: 'packages/open-desktop-skills/package.json',
                appPackageJson: 'packages/open-desktop/package.json',
                cliEntry: 'packages/open-desktop/dist/cli.js',
                commandName: 'open-desktop',
                requiredSections: ['Safety'],
            },
        ],
    },
    release: {
        registry: 'https://registry.npmjs.org',
        tagPattern: '^v(\\d+\\.\\d+\\.\\d+)$',
        packages: [
            { name: '@wae/open-desktop', dir: 'packages/open-desktop' },
            { name: '@wae/open-desktop-skills', dir: 'packages/open-desktop-skills' },
            { name: '@wae/open-desktop-win32-x64', dir: 'packages/open-desktop-win32-x64' },
        ],
    },
    ci: {
        repo: 'nyar-os/OpenDesktop',
        workflows: ['check-rust.yml', 'check-typescript.yml', 'publish-npm.yml'],
        requiredJobs: ['check-rust', 'check-typescript'],
        shaBinding: true,
    },
    install: {
        shortcutDefault: true,
    },
    audit: {
        actor: { kind: 'flag', id: 'flag', source: 'flag' },
    },
};
