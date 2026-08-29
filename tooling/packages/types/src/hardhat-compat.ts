import { createRequire } from "module";

export type HardhatTaskBuilderCompat = {
  addParam?: (name: string, description?: string) => any;
  addOptionalParam?: (
    name: string,
    description?: string,
    defaultValue?: any,
    type?: any,
  ) => any;
  addFlag?: (name: string, description?: string) => any;
  addOption?: (config: {
    name: string;
    description?: string;
    type?: string;
    defaultValue: any;
  }) => any;
  setAction?: (action: (...args: any[]) => any) => any;
  setInlineAction?: (action: (...args: any[]) => any) => any;
};

const ARG_TYPE_STRING = "STRING";
const ARG_TYPE_STRING_WITHOUT_DEFAULT = "STRING_WITHOUT_DEFAULT";
const ARG_TYPE_INT = "INT";
const ARG_TYPE_FLAG = "FLAG";
const hardhatCompatRequire = createRequire(__filename);

function getHardhat2ArgumentType(typeName: "int"): any {
  try {
    // Hardhat 2 exposes CLI param types via `hardhat/config`; Hardhat 3 doesn't.
    const hardhatConfig = hardhatCompatRequire("hardhat/config");
    return hardhatConfig?.types?.[typeName] ?? ARG_TYPE_INT;
  } catch {
    return ARG_TYPE_INT;
  }
}

function missingBuilderMethod(method: string): never {
  throw new Error(`Hardhat task builder does not support ${method}`);
}

export function addRequiredStringOption<T extends HardhatTaskBuilderCompat>(
  builder: T,
  name: string,
  description?: string,
): T {
  if (typeof builder.addParam === "function") {
    builder.addParam(name, description);
    return builder;
  }
  if (typeof builder.addOption === "function") {
    builder.addOption({
      name,
      description,
      type: ARG_TYPE_STRING_WITHOUT_DEFAULT,
      defaultValue: undefined,
    });
    return builder;
  }
  return missingBuilderMethod("required string options");
}

export function addOptionalStringOption<T extends HardhatTaskBuilderCompat>(
  builder: T,
  name: string,
  description?: string,
  defaultValue?: string,
): T {
  if (typeof builder.addOptionalParam === "function") {
    builder.addOptionalParam(name, description, defaultValue);
    return builder;
  }
  if (typeof builder.addOption === "function") {
    builder.addOption({
      name,
      description,
      type: defaultValue === undefined ? ARG_TYPE_STRING_WITHOUT_DEFAULT : ARG_TYPE_STRING,
      defaultValue,
    });
    return builder;
  }
  return missingBuilderMethod("optional string options");
}

export function addOptionalIntOption<T extends HardhatTaskBuilderCompat>(
  builder: T,
  name: string,
  description: string,
  defaultValue: number,
): T {
  if (typeof builder.addOptionalParam === "function") {
    builder.addOptionalParam(
      name,
      description,
      defaultValue,
      getHardhat2ArgumentType("int"),
    );
    return builder;
  }
  if (typeof builder.addOption === "function") {
    builder.addOption({
      name,
      description,
      type: ARG_TYPE_INT,
      defaultValue,
    });
    return builder;
  }
  return missingBuilderMethod("optional int options");
}

export function addFlagOption<T extends HardhatTaskBuilderCompat>(
  builder: T,
  name: string,
  description?: string,
): T {
  if (typeof builder.addParam === "function" && typeof builder.addFlag === "function") {
    builder.addFlag(name, description);
    return builder;
  }
  if (typeof builder.addOption === "function") {
    builder.addOption({
      name,
      description,
      type: ARG_TYPE_FLAG,
      defaultValue: false,
    });
    return builder;
  }
  if (typeof builder.addFlag === "function") {
    builder.addFlag({
      name,
      description,
    } as any);
    return builder;
  }
  return missingBuilderMethod("flag options");
}

export function setTaskAction<T extends HardhatTaskBuilderCompat>(
  builder: T,
  action: (...args: any[]) => any,
): T {
  if (typeof builder.setInlineAction === "function") {
    builder.setInlineAction(action);
    return builder;
  }
  if (typeof builder.setAction === "function") {
    builder.setAction(action);
    return builder;
  }
  return missingBuilderMethod("task actions");
}

export function getHardhatSelectedNetworkName(hre: any, explicitName?: string): string {
  if (typeof explicitName === "string" && explicitName.length > 0) {
    return explicitName;
  }

  const runtimeNetworkName =
    typeof hre?.network?.name === "string" && hre.network.name.length > 0
      ? hre.network.name
      : undefined;
  if (runtimeNetworkName) {
    return runtimeNetworkName;
  }

  const globalNetworkName =
    typeof hre?.globalOptions?.network === "string" && hre.globalOptions.network.length > 0
      ? hre.globalOptions.network
      : undefined;
  if (globalNetworkName) {
    return globalNetworkName;
  }

  const defaultNetworkName =
    typeof hre?.config?.defaultNetwork === "string" && hre.config.defaultNetwork.length > 0
      ? hre.config.defaultNetwork
      : undefined;

  return defaultNetworkName || "hardhat";
}
