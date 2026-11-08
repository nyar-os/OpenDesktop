/**
 * Typed library surface.
 *
 * Every function forwards to the Rust `napi` crate. The library never fabricates
 * capability state and never implements a protocol backend.
 */
import { loadNative, type AuditEntry, type AuditFilter, type CapabilitiesReport } from './native.js';

export type { AuditEntry, AuditFilter, CapabilitiesReport, Capability, CapabilityReason, ProtocolCapability } from './native.js';

/** Native addon version. */
export function version(): string {
    return loadNative().version();
}

/** Capability snapshot of the current host. */
export function capabilities(): CapabilitiesReport {
    return loadNative().capabilities();
}

/** Replace audit attribution for the current process. */
export function configureAudit(options: { actorKind?: string; actorId?: string; actorSource?: string; session?: string; host?: string }): void {
    loadNative().configureAudit(options);
}

/** Read audit records matching the filter. */
export function auditQuery(filter: AuditFilter = {}): AuditEntry[] {
    return loadNative().auditQuery(filter);
}

/** Open the window and return the backend name. */
export function openGui(options: { contentDir?: string; title?: string } = {}): string {
    return loadNative().openGui(options).backend;
}

/** Pump pending window messages; returns whether the window is still open. */
export function pumpGui(): boolean {
    return loadNative().pumpGui();
}

/** Close the window; returns whether a window was closed. */
export function closeGui(): boolean {
    return loadNative().closeGui();
}

/** Whether a window is currently open. */
export function guiStatus(): boolean {
    return loadNative().guiStatus();
}

/** Open the window, pump host events until it closes, and return the backend name. */
export async function runGui(options: { contentDir?: string; title?: string } = {}): Promise<string> {
    const native = loadNative();
    const { backend } = native.openGui(options);
    try {
        while (native.pumpGui()) {
            await new Promise((resolve) => setTimeout(resolve, 16));
        }
    }
    finally {
        native.closeGui();
    }
    return backend;
}
