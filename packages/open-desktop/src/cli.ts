#!/usr/bin/env node
/**
 * Agent-facing CLI.
 *
 * Contract:
 * - non-interactive, always exits with a stable code
 * - `--json` returns `{ ok, data }` or `{ ok: false, error }`
 * - every command configures the audit actor before calling the native layer
 * - only the real capability surface is exposed: no session CRUD
 * - exit codes: 0 success, 1 runtime failure, 2 usage, 3 capability unavailable
 */
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { createCli, type CommandContext, type ParsedOptions } from '@wae/commander';

import { AppError, EXIT, fromNativeError } from './errors.js';
import {
    auditQuery,
    capabilities,
    configureAudit,
    runGui,
    version,
    type AuditEntry,
    type CapabilitiesReport,
    type Capability,
    type CapabilityReason,
} from './index.js';

const here = path.dirname(fileURLToPath(import.meta.url));
const localesDir = path.join(here, '..', 'locales');

const cli = createCli('open-desktop')
    .locales(localesDir)
    .intro('cli.intro')
    .option('--actor <id>', 'cli.opt.actor')
    .option('--actor-kind <kind>', 'cli.opt.actor-kind')
    .option('--actor-source <source>', 'cli.opt.actor-source')
    .option('--json', 'cli.opt.json');

function stringOption(options: ParsedOptions, key: string): string | undefined {
    const value = options[key];
    return typeof value === 'string' && value.trim() ? value : undefined;
}

function flag(options: ParsedOptions, key: string): boolean {
    return options[key] === true;
}

function integerOption(options: ParsedOptions, key: string): number | undefined {
    const raw = stringOption(options, key);
    if (raw === undefined) return undefined;
    const value = Number(raw);
    if (!Number.isInteger(value) || value < 0) {
        throw new AppError({ code: 'invalid_argument', message: `\`--${key}\` must be a non-negative integer` });
    }
    return value;
}

/** Configure the audit actor before any capability call. */
function bootstrap(options: ParsedOptions): void {
    if (options._.length) {
        throw new AppError({ code: 'invalid_argument', message: `unexpected argument \`${options._.join(' ')}\`, every input is a named option` });
    }
    const kind = stringOption(options, 'actor-kind');
    if (kind !== undefined && kind !== 'user' && kind !== 'agent') {
        throw new AppError({ code: 'invalid_argument', message: '`--actor-kind` must be `user` or `agent`' });
    }
    configureAudit({
        actorKind: kind === 'agent' ? 'agent' : 'user',
        actorId: stringOption(options, 'actor') ?? 'unknown',
        actorSource: stringOption(options, 'actor-source') ?? 'cli',
    });
}

function emit(options: ParsedOptions, ctx: CommandContext, data: unknown, human: string): void {
    if (flag(options, 'json')) {
        ctx.stdout(JSON.stringify({ ok: true, data }, null, 2));
        return;
    }
    if (human) ctx.stdout(human);
}

function fail(options: ParsedOptions, ctx: CommandContext, error: unknown): number {
    const appError = fromNativeError(error);
    if (flag(options, 'json')) {
        ctx.stdout(JSON.stringify({ ok: false, error: appError.toJSON() }, null, 2));
    }
    else {
        ctx.stderr(`${appError.code}: ${appError.message}`);
    }
    return appError.exitCode;
}

function reasonText(reason: CapabilityReason | undefined): string {
    if (!reason) return 'unavailable';
    switch (reason.code) {
        case 'unsupported_platform':
            return reason.detail ? `unsupported on ${reason.platform}: ${reason.detail}` : `unsupported on ${reason.platform ?? 'this platform'}`;
        case 'dependency_blocked':
            return `blocked on ${reason.dependency ?? 'an external dependency'}${reason.detail ? `: ${reason.detail}` : ''}`;
        case 'not_implemented':
            return `${reason.scope ?? 'this backend'} is not implemented yet`;
        default:
            return reason.code;
    }
}

function describe(name: string, capability: Capability): string {
    if (capability.available) return `${name}: available`;
    return `${name}: ${capability.reason?.code ?? 'unavailable'} — ${reasonText(capability.reason)}`;
}

function formatCapabilities(report: CapabilitiesReport): string {
    return [
        `platform: ${report.platform}`,
        describe('gui', report.gui),
        ...report.protocols.map((entry) => describe(`protocol ${entry.protocol}`, entry)),
        describe('credentials', report.credentials),
    ].join('\n');
}

function formatAudit(entries: AuditEntry[]): string {
    if (!entries.length) return '(no audit records)';
    return entries
        .map((entry) => {
            const actor = `${entry.actor.kind}:${entry.actor.id}@${entry.actor.source}`;
            const detail = entry.errorCode ? ` error=${entry.errorCode}` : '';
            return `${new Date(entry.timestampMs).toISOString()}\t${entry.capability}\t${entry.result}\t${actor}${detail}\t${entry.targets.summary}`;
        })
        .join('\n');
}

cli.command('capabilities', 'cli.cmd.capabilities')
    .actionCtx(async (options, _args, ctx) => {
        try {
            bootstrap(options);
            const report = capabilities();
            emit(options, ctx, report, formatCapabilities(report));
            return EXIT.OK;
        }
        catch (error) {
            return fail(options, ctx, error);
        }
    });

cli.command('version', 'cli.cmd.version')
    .actionCtx(async (options, _args, ctx) => {
        try {
            bootstrap(options);
            const current = version();
            emit(options, ctx, { version: current }, current);
            return EXIT.OK;
        }
        catch (error) {
            return fail(options, ctx, error);
        }
    });

cli.command('audit', 'cli.cmd.audit')
    .option('--kind <kind>', 'cli.opt.kind')
    .option('--id <id>', 'cli.opt.id')
    .option('--capability <name>', 'cli.opt.capability')
    .option('--from-ms <n>', 'cli.opt.from-ms')
    .option('--to-ms <n>', 'cli.opt.to-ms')
    .option('--limit <n>', 'cli.opt.limit')
    .actionCtx(async (options, _args, ctx) => {
        try {
            bootstrap(options);
            const kind = stringOption(options, 'kind');
            if (kind !== undefined && kind !== 'user' && kind !== 'agent') {
                throw new AppError({ code: 'invalid_argument', message: '`--kind` must be `user` or `agent`' });
            }
            const entries = auditQuery({
                kind: kind === 'agent' ? 'agent' : kind === 'user' ? 'user' : undefined,
                id: stringOption(options, 'id'),
                capability: stringOption(options, 'capability'),
                fromMs: integerOption(options, 'from-ms'),
                toMs: integerOption(options, 'to-ms'),
                limit: integerOption(options, 'limit'),
            });
            emit(options, ctx, entries, formatAudit(entries));
            return EXIT.OK;
        }
        catch (error) {
            return fail(options, ctx, error);
        }
    });

cli.command('gui', 'cli.cmd.gui')
    .option('--content-dir <path>', 'cli.opt.content-dir')
    .actionCtx(async (options, _args, ctx) => {
        try {
            bootstrap(options);
            const backend = await runGui({ contentDir: stringOption(options, 'content-dir') });
            emit(options, ctx, { backend }, `closed (${backend})`);
            return EXIT.OK;
        }
        catch (error) {
            return fail(options, ctx, error);
        }
    });

const code = await cli.parse(process.argv);
process.exitCode = code;
