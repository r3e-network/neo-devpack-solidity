import { describe, expect, it, vi } from "vitest";
import {
  addFlagOption,
  addOptionalIntOption,
  addOptionalStringOption,
  addRequiredStringOption,
  getHardhatSelectedNetworkName,
  setTaskAction,
} from "../src/hardhat-compat";

describe("hardhat task compatibility helpers", () => {
  it("uses Hardhat 2-style builder methods when present", () => {
    const builder = {
      addParam: vi.fn().mockReturnThis(),
      addOptionalParam: vi.fn().mockReturnThis(),
      addFlag: vi.fn().mockReturnThis(),
      setAction: vi.fn().mockReturnThis(),
    };
    const action = vi.fn();

    addRequiredStringOption(builder, "contract", "Contract name");
    addOptionalStringOption(builder, "args", "Constructor arguments", "[]");
    addOptionalIntOption(builder, "waitBlocks", "Blocks to wait", 5);
    addFlagOption(builder, "verify", "Verify after deploy");
    setTaskAction(builder, action);

    expect(builder.addParam).toHaveBeenCalledWith("contract", "Contract name");
    expect(builder.addOptionalParam).toHaveBeenCalledWith("args", "Constructor arguments", "[]");
    const waitBlocksCall = builder.addOptionalParam.mock.calls.find(
      ([name]) => name === "waitBlocks",
    );
    expect(waitBlocksCall).toBeDefined();
    expect(waitBlocksCall?.slice(0, 3)).toEqual(["waitBlocks", "Blocks to wait", 5]);
    expect(
      waitBlocksCall?.[3] === "INT" || waitBlocksCall?.[3]?.name === "int",
    ).toBe(true);
    expect(builder.addFlag).toHaveBeenCalledWith("verify", "Verify after deploy");
    expect(builder.setAction).toHaveBeenCalledWith(action);
  });

  it("uses Hardhat 3-style builder methods when present", () => {
    const builder = {
      addOption: vi.fn().mockReturnThis(),
      setInlineAction: vi.fn().mockReturnThis(),
    };
    const action = vi.fn();

    addRequiredStringOption(builder, "contract", "Contract name");
    addOptionalStringOption(builder, "args", "Constructor arguments", "[]");
    addOptionalIntOption(builder, "waitBlocks", "Blocks to wait", 5);
    addFlagOption(builder, "verify", "Verify after deploy");
    setTaskAction(builder, action);

    expect(builder.addOption).toHaveBeenCalledWith({
      name: "contract",
      description: "Contract name",
      type: "STRING_WITHOUT_DEFAULT",
      defaultValue: undefined,
    });
    expect(builder.addOption).toHaveBeenCalledWith({
      name: "args",
      description: "Constructor arguments",
      type: "STRING",
      defaultValue: "[]",
    });
    expect(builder.addOption).toHaveBeenCalledWith({
      name: "waitBlocks",
      description: "Blocks to wait",
      type: "INT",
      defaultValue: 5,
    });
    expect(builder.addOption).toHaveBeenCalledWith({
      name: "verify",
      description: "Verify after deploy",
      type: "FLAG",
      defaultValue: false,
    });
    expect(builder.setInlineAction).toHaveBeenCalledWith(action);
  });

  it("derives the selected network name from explicit, runtime, or global options", () => {
    expect(getHardhatSelectedNetworkName({ network: { name: "neo_testnet" } })).toBe("neo_testnet");
    expect(getHardhatSelectedNetworkName({ globalOptions: { network: "neo_mainnet" } })).toBe("neo_mainnet");
    expect(getHardhatSelectedNetworkName({ config: { defaultNetwork: "default-net" } })).toBe("default-net");
    expect(getHardhatSelectedNetworkName({}, "explicit-net")).toBe("explicit-net");
  });
});
