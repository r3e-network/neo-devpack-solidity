/**
 * Source location attached to a build diagnostic.
 */
export interface SourceLocation {
  file?: string;
  start?: number;
  end?: number;
}

/**
 * Normalized diagnostic emitted during `neo-forge build`.
 */
export interface BuildDiagnostic {
  severity: "error" | "warning" | "info";
  message: string;
  code?: string;
  sourceLocation?: SourceLocation;
  formattedMessage?: string;
}

/**
 * Error thrown when `neo-forge build` fails.
 *
 * Carries normalized diagnostics so the CLI can print readable source locations
 * and forward compiler error codes such as `NSH-XXXX`.
 */
export class NeoForgeBuildError extends Error {
  public readonly code: string;
  public readonly diagnostics: BuildDiagnostic[];
  public readonly exitCode?: number;

  constructor(
    message: string,
    opts: { code?: string; diagnostics?: BuildDiagnostic[]; exitCode?: number } = {}
  ) {
    super(message);
    this.name = "NeoForgeBuildError";
    this.code = opts.code ?? "NSH-7001";
    this.diagnostics = opts.diagnostics ?? [];
    this.exitCode = opts.exitCode;
  }
}
