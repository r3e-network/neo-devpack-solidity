import { spawn } from "child_process";
import { accessSync, constants, promises as fs } from "fs";
import path from "path";
import type { CompilationInput, CompilationOutput, CompilationError } from "@neo-devpack-solidity/types";
import { NeoForgeBuildError, BuildDiagnostic } from "./build-error.js";

export interface CompilerInvokerOptions {
  /** Path to the `neo-solc` binary. Defaults to auto-discovery. */
  neoSolcPath?: string;
  /** Working directory used for temporary input/output files. */
  cwd?: string;
  /** Forward compiler stdout/stderr to the parent process. */
  verbose?: boolean;
}

/**
 * Invokes `neo-solc --standard-json` with a Solidity Standard JSON input.
 *
 * The invoker is intentionally a thin wrapper around the compiler binary: it
 * does not re-implement compilation, only constructs the input, spawns the
 * process, and normalizes the output/errors.
 */
export class CompilerInvoker {
  /**
   * Run `neo-solc` with the given Standard JSON input.
   */
  async invoke(input: CompilationInput, opts: CompilerInvokerOptions = {}): Promise<CompilationOutput> {
    const bin = opts.neoSolcPath || this.resolveNeoSolc();
    const cwd = opts.cwd || process.cwd();
    const workDir = path.join(cwd, ".neo-forge");
    const inputFile = path.join(workDir, "neo-solc-input.json");
    const outputFile = path.join(workDir, "neo-solc-output.json");

    await fs.mkdir(workDir, { recursive: true });
    await fs.writeFile(inputFile, JSON.stringify(input, null, 2));

    const exitCode = await this.runNeoSolc(bin, inputFile, outputFile, opts.verbose);

    let outputContent: string;
    try {
      outputContent = await fs.readFile(outputFile, "utf-8");
    } catch (error) {
      throw new NeoForgeBuildError(
        `neo-solc produced no output (exit code ${exitCode ?? "unknown"}).`,
        { code: "NSH-7002", exitCode: exitCode ?? undefined }
      );
    }

    let output: CompilationOutput;
    try {
      output = JSON.parse(outputContent) as CompilationOutput;
    } catch {
      throw new NeoForgeBuildError(
        "neo-solc produced invalid JSON output.",
        { code: "NSH-7003", exitCode: exitCode ?? undefined }
      );
    }

    this.validateOutput(output, exitCode ?? undefined);
    return output;
  }

  /**
   * Resolve the `neo-solc` binary using the `NEO_SOLC` environment variable or
   * common build artifact locations.
   */
  private resolveNeoSolc(): string {
    const ext = process.platform === "win32" ? ".exe" : "";
    const fromEnv = process.env.NEO_SOLC;

    const candidates = [
      fromEnv,
      path.join(process.cwd(), `bin/neo-solc${ext}`),
      path.join(process.cwd(), `target/release/neo-solc${ext}`),
      path.join(process.cwd(), `target/debug/neo-solc${ext}`),
      path.resolve(process.cwd(), `../target/release/neo-solc${ext}`),
      path.resolve(process.cwd(), `../target/debug/neo-solc${ext}`),
      path.resolve(process.cwd(), `../../target/release/neo-solc${ext}`),
      path.resolve(process.cwd(), `../../target/debug/neo-solc${ext}`),
    ].filter((c): c is string => Boolean(c));

    for (const candidate of candidates) {
      try {
        accessSync(candidate, constants.X_OK);
        return candidate;
      } catch {
        // continue
      }
    }

    return "neo-solc";
  }

  private runNeoSolc(
    bin: string,
    inputFile: string,
    outputFile: string,
    verbose?: boolean
  ): Promise<number | null> {
    return new Promise((resolve, reject) => {
      const child = spawn(bin, ["--standard-json", "--input", inputFile, "--output", outputFile], {
        stdio: verbose ? ["ignore", "inherit", "inherit"] : ["ignore", "pipe", "pipe"],
        env: { ...process.env },
      });

      let stderr = "";
      if (!verbose) {
        child.stderr?.on("data", (data) => {
          stderr += data.toString();
        });
      }

      child.on("error", (error) => {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") {
          reject(
            new NeoForgeBuildError(
              `neo-solc binary not found ('${bin}'). Build it with ` +
                "`cargo build --release --bin neo-solc` and put it on PATH, " +
                "or set the NEO_SOLC environment variable to its path.",
              { code: "NSH-7004" }
            )
          );
        } else {
          reject(
            new NeoForgeBuildError(
              `Failed to start neo-solc: ${error.message}`,
              { code: "NSH-7005" }
            )
          );
        }
      });

      child.on("close", (code) => {
        resolve(code);
      });
    });
  }

  private validateOutput(output: CompilationOutput, exitCode?: number): void {
    const diagnostics = (output.errors ?? []).map(this.toBuildDiagnostic);
    const errors = diagnostics.filter((d) => d.severity === "error");

    if (errors.length > 0 || (exitCode !== undefined && exitCode !== 0)) {
      const first = errors[0] ?? diagnostics[0];
      const location = first?.sourceLocation;
      const locationSuffix = location?.file
        ? ` at ${location.file}${location.start !== undefined ? `:${location.start}` : ""}`
        : "";
      const code = first?.code ?? "NSH-7006";
      throw new NeoForgeBuildError(
        `${code}: neo-solc compilation failed${locationSuffix}: ${first?.message ?? "unknown error"}`,
        { code, diagnostics, exitCode }
      );
    }

    if (!output.contracts || Object.keys(output.contracts).length === 0) {
      throw new NeoForgeBuildError(
        "NSH-7007: neo-solc produced no contracts.",
        { code: "NSH-7007", diagnostics, exitCode }
      );
    }
  }

  private toBuildDiagnostic(error: CompilationError): BuildDiagnostic {
    return {
      severity: error.severity,
      message: error.message,
      code: error.code,
      formattedMessage: error.formattedMessage,
      sourceLocation: error.sourceLocation
        ? {
            file: error.sourceLocation.file,
            start: error.sourceLocation.start,
            end: error.sourceLocation.end,
          }
        : undefined,
    };
  }
}
