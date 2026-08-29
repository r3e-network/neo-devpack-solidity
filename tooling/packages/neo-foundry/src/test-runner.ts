export interface TestRunnerOptions {
  pattern?: string;
  watch?: boolean;
  gasReport?: boolean;
}

export class TestRunner {
  async run(options: TestRunnerOptions = {}): Promise<never> {
    const label = options.pattern ?? "all tests";
    throw new Error(
      `Neo Foundry test runner is not implemented. Cannot run ${label}.`
    );
  }
}
