declare module "hardhat/config" {
  export const types: {
    boolean: any;
    int: any;
    string: any;
  };

  interface TaskDefinition {
    addParam(name: string, description?: string): TaskDefinition;
    addOptionalParam(name: string, description?: string, defaultValue?: any, type?: any): TaskDefinition;
    addFlag(name: string, description?: string): TaskDefinition;
    setAction(action: (...args: any[]) => any): TaskDefinition;
  }

  export function task(name: string, description?: string): TaskDefinition;
  export function extendConfig(action: (...args: any[]) => any): void;
  export function extendEnvironment(action: (...args: any[]) => any): void;
}

declare module "hardhat/plugins" {
  export class HardhatPluginError extends Error {
    constructor(pluginName?: string, message?: string);
  }

  export function lazyObject<T>(fn: () => T): T;
}

declare module "hardhat/types" {
  export type TaskArguments = Record<string, any>;
  export type HardhatConfig = any;
  export type HardhatUserConfig = any;

  export interface HardhatRuntimeEnvironment {
    config: any;
    network: { name: string };
    artifacts: any;
    run(taskName: string, taskArgs?: any): Promise<any>;
    neoDeploy: any;
    neoSolc: any;
  }
}

declare module "hardhat/types/config" {
  export interface HardhatUserConfig {
    [key: string]: any;
  }

  export interface HardhatConfig {
    [key: string]: any;
  }
}

declare module "hardhat/types/runtime" {
  export interface HardhatRuntimeEnvironment {
    [key: string]: any;
  }
}
