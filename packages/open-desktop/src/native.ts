/**
 * Lazy loader for the platform-native addon.
 *
 * The addon is an optional dependency. It is loaded only when a native
 * capability is called, so `--help`, `version`, and locale resolution work on
 * machines without the binary. There is no JavaScript fallback: a missing or
 * unloadable addon is a structured `native_unavailable` failure.
 */
import { createRequire } from 'node:module';

import { AppError } from './errors.js';

const require = createRequire(import.meta.url);
const PLATFORM_PACKAGE = '@wae/open-desktop-win32-x64';

/** Structured reason reported by the Rust capability snapshot. */
export interface CapabilityReason {
    /** `unsupported_platform`, `not_implemented`, or `dependency_blocked`. */
    code: string;
    /** Platform tag for `unsupported_platform`. */
    platform?: string;
    /** Upstream provider detail for `unsupported_platform`. */
    detail?: string;
    /** Scope name for `not_implemented`. */
    scope?: string;
    /** Dependency name for `dependency_blocked`. */
    dependency?: string;
}

/** Availability of one capability. */
export interface Capability {
    available: boolean;
    reason?: CapabilityReason;
}

/** Availability of one protocol backend. */
export interface ProtocolCapability extends Capability {
    protocol: 'ssh' | 'rdp' | 'vnc';
}

/** Capability snapshot of the current host. */
export interface CapabilitiesReport {
    platform: string;
    gui: Capability;
    protocols: ProtocolCapability[];
    credentials: Capability;
}

/** One shared audit record (`wae.audit/v0`). */
export interface AuditEntry {
    schema: string;
    timestampMs: number;
    actor: { kind: 'user' | 'agent'; id: string; source: string };
    app: string;
    appVersion: string;
    capability: string;
    params: Record<string, unknown>;
    mode: 'dry_run' | 'execute';
    targets: { count: number; summary: string };
    result: 'success' | 'failure' | 'rejected';
    errorCode: string | null;
    durationMs: number;
    host: string;
    session: string;
}

/** Actor reported to the audit trail. */
export interface AuditActorOptions {
    actorKind?: string;
    actorId?: string;
    actorSource?: string;
    session?: string;
    host?: string;
}

/** Audit query filter. */
export interface AuditFilter {
    kind?: 'user' | 'agent';
    id?: string;
    capability?: string;
    fromMs?: number;
    toMs?: number;
    limit?: number;
}

/** Raw surface exported by the Rust `napi` crate. */
export interface NativeBindings {
    version(): string;
    capabilities(): CapabilitiesReport;
    configureAudit(options: AuditActorOptions): { ok: boolean };
    auditQuery(filter?: AuditFilter): AuditEntry[];
    openGui(options?: { contentDir?: string; title?: string }): { backend: string };
    pumpGui(): boolean;
    closeGui(): boolean;
    guiStatus(): boolean;
}

let cached: NativeBindings | null = null;

/** Load the native addon once; failures are structured and never fall back. */
export function loadNative(): NativeBindings {
    if (cached) return cached;
    try {
        cached = require(PLATFORM_PACKAGE) as NativeBindings;
        return cached;
    }
    catch (error) {
        cached = null;
        const message = error instanceof Error ? error.message : String(error);
        throw new AppError({
            code: 'native_unavailable',
            platform: `${process.platform}-${process.arch}`,
            message:
                `cannot load ${PLATFORM_PACKAGE}: ${message}\n` +
                'Run `pnpm run build:napi` from the repository root, ' +
                'or install the matching platform package.',
        });
    }
}
