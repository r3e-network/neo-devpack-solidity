import { promises as fs } from "fs";
import path from "path";
import type { CompilationOutput, CompiledContract } from "@neo-devpack-solidity/types";
import { NeoForgeBuildError } from "./build-error.js";

export interface NeoForgeBuildArtifact {
  contractName: string;
  sourceName: string;
  nefPath: string;
  manifestPath: string;
}

/**
 * Collects Neo-specific compilation artifacts from `neo-solc` Standard JSON
 * output and writes them to the configured `out/` directory.
 */
export class ArtifactCollector {
  /**
   * Write `.nef` and `.manifest.json` artifacts for every compiled contract.
   */
  async collect(output: CompilationOutput, outDir: string): Promise<NeoForgeBuildArtifact[]> {
    await fs.mkdir(outDir, { recursive: true });
    const artifacts: NeoForgeBuildArtifact[] = [];

    for (const [sourceName, contracts] of Object.entries(output.contracts)) {
      for (const [contractName, contract] of Object.entries(contracts)) {
        const artifact = await this.writeContractArtifact(
          sourceName,
          contractName,
          contract as CompiledContract,
          outDir
        );
        if (artifact) {
          artifacts.push(artifact);
        }
      }
    }

    if (artifacts.length === 0) {
      throw new NeoForgeBuildError(
        "NSH-7008: no Neo artifacts were emitted by neo-solc.",
        { code: "NSH-7008" }
      );
    }

    return artifacts;
  }

  private async writeContractArtifact(
    sourceName: string,
    contractName: string,
    contract: CompiledContract,
    outDir: string
  ): Promise<NeoForgeBuildArtifact | null> {
    const neo = contract.neo;
    if (!neo?.nef || !neo?.manifest) {
      return null;
    }

    const contractOutDir = path.join(outDir, path.dirname(sourceName), contractName);
    await fs.mkdir(contractOutDir, { recursive: true });

    const nefPath = path.join(contractOutDir, `${contractName}.nef`);
    const manifestPath = path.join(contractOutDir, `${contractName}.manifest.json`);

    const image = neo.nef.image || "";
    if (!image) {
      throw new NeoForgeBuildError(
        `NSH-7009: missing NEF image for contract ${contractName}.`,
        { code: "NSH-7009" }
      );
    }

    await fs.writeFile(nefPath, Buffer.from(image, "hex"));
    await fs.writeFile(manifestPath, JSON.stringify(neo.manifest, null, 2));

    return {
      contractName,
      sourceName,
      nefPath,
      manifestPath,
    };
  }
}
