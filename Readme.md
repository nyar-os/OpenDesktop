# Open Desktop

Remote session manager for the Open family. SSH sessions are first-class, so machines without a graphical desktop are managed the same way as desktop ones.

## Status

The repository is being migrated from its template scaffold to the shared Open application layout. Nothing here is shipped yet: the Rust workspace, npm packages, CLI and GUI land in later steps.

## Scope

- Session types: SSH first. RDP and VNC entries stay reserved and report an explicit unavailable state until their own backends are delivered.
- Platform: `win32-x64` first. Other platforms stay unsupported until a matching platform package exists.
- Credentials: stored through the operating system facility. There is no plaintext fallback.

## Target layout

- `crates/core`: session model, capability reporting, platform traits.
- `crates/napi`: Node-API surface.
- `packages/open-desktop`: npm meta package with native loader and CLI.
- `packages/open-desktop-ui`: VMZ static UI hosted in WebView2.
- `packages/open-desktop-win32-x64`: native platform package.

## Build

```bash
cargo check --workspace
cargo test --workspace
pnpm install
pnpm run build
```

## License

Apache License 2.0. See [License.md](./License.md).
