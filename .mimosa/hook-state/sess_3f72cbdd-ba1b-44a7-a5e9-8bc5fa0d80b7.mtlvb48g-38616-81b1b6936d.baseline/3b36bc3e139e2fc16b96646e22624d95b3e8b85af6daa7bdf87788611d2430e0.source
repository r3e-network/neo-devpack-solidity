export interface DeployOptions {
  contract?: string;
  constructorArgs?: string[];
  network?: string;
}

export class NeoFoundryDeployer {
  async deploy(options: DeployOptions = {}): Promise<never> {
    const contract = options.contract ?? "<unspecified>";
    throw new Error(
      `Neo Foundry deployer is not implemented. Cannot deploy ${contract}.`
    );
  }
}
