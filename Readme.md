# OpenDesktop

Capability-gated desktop connectivity surface for the Open family. This repository
reports what the host can actually do and refuses everything else with stable
codes. It does not create sessions, store credentials, or implement SSH, RDP,
VNC, SFTP, or port forwarding.

## Status

The shared Open application layout is in place:

- Rust workspace: `crates/core` and `crates/napi`.
- npm packages: meta CLI, agent skills, VMZ UI, and the `win32-x64` native addon.
- Single capability face: the CLI, the GUI page, and the skills consume only the
  Node-API exports, and every export writes through `wae.audit/v0`.

## Capabilities

| Capability | State | Meaning |
| --- | --- | --- |
| SSH | `dependency_blocked` | Transport dependency selection and review are pending |
| RDP | `not_implemented` | No backend |
| VNC | `not_implemented` | No backend |
| Credentials | `not_implemented` on Windows, `unsupported_platform` elsewhere | No credential store binding |
| GUI | available on `win32-x64` only | Host window for the staged VMZ build |

`open-desktop capabilities --json` is the only capability source. Do not infer
availability from platform names or from this table.

## Layout

| Path | Purpose |
| --- | --- |
| `crates/core` | Capability snapshot, session model types, structured errors |
| `crates/napi` | Node-API exports, audit boundary, GUI host lifecycle |
| `packages/open-desktop` | `@wae/open-desktop` typed library and `open-desktop` CLI |
| `packages/open-desktop-skills` | `@wae/open-desktop-skills` agent skill |
| `packages/open-desktop-ui` | `@wae/open-desktop-ui` VMZ static UI, staged into `packages/open-desktop/dist/gui` |
| `packages/open-desktop-win32-x64` | `@wae/open-desktop-win32-x64` native addon |
| `scripts` | Native build/staging, skills validation |
| `wae.config.ts` | Build, skills, release, CI, and audit configuration |

## CLI

```bash
open-desktop capabilities --json
open-desktop version --json
open-desktop audit --json --limit 20 --kind agent
open-desktop gui --content-dir ./dist/gui
```

Exit codes: `0` success, `1` runtime failure, `2` usage error, `3` capability
unavailable. Audit actor flags are `--actor`, `--actor-kind`, and `--actor-source`.

## Build and verify

Requirements: Node.js 22 or newer, pnpm 10.28, and the Rust toolchain pinned in
`rust-toolchain.toml`. The native addon currently targets `win32-x64` only.

```bash
pnpm install
pnpm run build
pnpm run check:skills
pnpm run check:rust
node packages/open-desktop/dist/cli.js capabilities --json
node packages/open-desktop/dist/cli.js version --json
node packages/open-desktop/dist/cli.js audit --json --limit 1
```

`pnpm run build` runs `build:napi`, `build:ts`, and `build:ui`. The UI is a VMZ
static delivery staged into the meta package, and it renders only the snapshot
returned by the host IPC handler.

## License

Apache License 2.0. See [License.md](./License.md).
