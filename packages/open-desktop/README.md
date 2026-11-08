# `@wae/open-desktop`

Typed library and agent CLI for OpenDesktop. Every call forwards to the Rust
Node-API addon; the package contains no protocol backend and no session storage.

## Install

```bash
npm install @wae/open-desktop @wae/open-desktop-win32-x64
```

The native addon is an optional dependency. When it cannot be loaded the CLI
fails with a structured `native_unavailable` error instead of falling back to
JavaScript.

## CLI

```bash
open-desktop capabilities --json
open-desktop version --json
open-desktop audit --json --limit 20 --kind agent
open-desktop gui --content-dir ./dist/gui
```

Exit codes: `0` success, `1` runtime failure, `2` usage, `3` capability
unavailable.

## Capabilities

| Capability | State |
| --- | --- |
| GUI | available on `win32-x64` only |
| SSH | `dependency_blocked` |
| RDP | `not_implemented` |
| VNC | `not_implemented` |
| Credentials | `not_implemented` on Windows, `unsupported_platform` elsewhere |

## Library

```ts
import { capabilities, auditQuery, runGui } from '@wae/open-desktop';

const report = capabilities();
const entries = auditQuery({ limit: 10 });
const backend = await runGui({ contentDir: './dist/gui' });
```

## License

Apache License 2.0. See [License.md](./License.md).
