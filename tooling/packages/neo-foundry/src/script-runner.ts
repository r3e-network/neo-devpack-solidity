export interface ScriptRunnerOptions {
  scriptPath?: string;
  args?: string[];
  env?: Record<string, string>;
}

/**
 * Placeholder script runner for Neo Foundry. The real implementation still lives
 * on the roadmap, so every call surfaces a consistent error rather than leaving
 * an untyped stub.
 */
export class ScriptRunner {
  async run(options: ScriptRunnerOptions = {}): Promise<never> {
    const script = options.scriptPath ?? "<unknown>";
    throw new Error(
      `Neo Foundry script runner is not implemented. Attempted to run ${script}.`
    );
  }
}
