---
name: open-desktop
description: Report OpenDesktop capability availability, read audit records, and open the local GUI through the `open-desktop` CLI. Use when inspecting desktop connectivity support or audit history.
---

# OpenDesktop

Use this skill only to inspect the real capability surface of OpenDesktop, read
its audit trail, or open the local window. Session creation, protocol transport,
credential storage, and port forwarding are not part of this repository yet.

## When to use

- Check whether SSH, RDP, VNC, credentials, or the GUI can be used on this host.
- Read audit records to see which capabilities ran and whether they succeeded.
- Open the local VMZ window for the staged builds.

## Prerequisites

- Node.js 22 or newer.
- The `open-desktop` CLI installed from the `@wae/open-desktop` package, or run
  from a checkout after `pnpm run build`.

## Commands

Read the capability snapshot as JSON:

`open-desktop capabilities --json`

Print the native addon version:

`open-desktop version --json`

Read the most recent audit records, optionally filtered:

`open-desktop audit --json --limit <limit> --kind <kind> --capability <capability>`

Open the local window for a staged GUI directory:

`open-desktop gui --content-dir <path>`

Every command accepts `--actor <id>`, `--actor-kind <kind>`, `--actor-source <source>`,
and `--json`. Actor values are recorded in the audit trail.

## Capability states

The snapshot reports one entry per capability and one entry per protocol:

- `dependency_blocked` for SSH: delivery waits on an external transport
  dependency and its review.
- `not_implemented` for RDP and VNC backends.
- `not_implemented` for credential storage on Windows, and
  `unsupported_platform` elsewhere.
- The GUI is available only where a matching platform package exists.

Treat every non-available state as a hard stop. Never promise that a blocked
protocol works, never invent a session, and do not ask the user to retry the same
blocked capability expecting a different result.

## Safety

- Read `open-desktop capabilities --json` before doing anything else, and respect
  the reported states.
- Confirm with the operator before running `open-desktop gui --content-dir <path>`,
  because it opens an interactive window.
- Never put secrets, passwords, private keys, or tokens on the command line or in
  audit filters.
- Audit records are append-only through the shared `wae.audit/v0` contract. Do not
  edit or truncate them.
