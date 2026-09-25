import { describe, expect, it } from "vitest";

import { createNeoDeployRuntime } from "../src/runtime-bootstrap";

describe("createNeoDeployRuntime", () => {
  it("defers missing neo network errors until deployment features are accessed", () => {
    const neoDeploy = createNeoDeployRuntime({
      config: {
        neoNetworks: {},
      },
      network: {
        name: "hardhat",
      },
      artifacts: {},
    } as any);

    expect(() => neoDeploy.accounts).toThrow(
      'Neo network configuration not found for "hardhat"',
    );
    expect(() => neoDeploy.rpc).toThrow(
      'Neo network configuration not found for "hardhat"',
    );
    expect(() => neoDeploy.deployer).toThrow(
      'Neo network configuration not found for "hardhat"',
    );
  });
});
