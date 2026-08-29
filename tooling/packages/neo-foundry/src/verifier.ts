export interface VerifyContractOptions {
  contractAddress: string;
  contractName?: string;
  constructorArgs?: string[];
  explorer?: string;
}

export class NeoFoundryVerifier {
  async verify(options: VerifyContractOptions): Promise<never> {
    throw new Error(
      `Neo Foundry verifier is not implemented. Cannot verify ${options.contractAddress}.`
    );
  }
}
