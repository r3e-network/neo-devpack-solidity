export interface CLICommand {
  name: string;
  description: string;
  aliases?: string[];
  options: CLIOption[];
  subcommands?: CLICommand[];
  action: (args: CLIArgs) => Promise<void>;
  examples?: CLIExample[];
}

export interface CLIOption {
  name: string;
  description: string;
  type: 'string' | 'number' | 'boolean' | 'array';
  required?: boolean;
  default?: any;
  choices?: string[];
  alias?: string;
  hidden?: boolean;
}

export interface CLIArgs {
  [key: string]: any;
  _: string[];
  $0: string;
}

export interface CLIExample {
  command: string;
  description: string;
}

export interface CLIContext {
  config: any;
  logger: CLILogger;
  spinner: CLISpinner;
  progress: CLIProgress;
  interactive: CLIInteractive;
}

export interface CLILogger {
  info(message: string): void;
  warn(message: string): void;
  error(message: string | Error): void;
  success(message: string): void;
  debug(message: string): void;
  log(level: LogLevel, message: string): void;
  setLevel(level: LogLevel): void;
}

export type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'trace';

export interface CLISpinner {
  start(text: string): void;
  succeed(text?: string): void;
  fail(text?: string): void;
  warn(text?: string): void;
  info(text?: string): void;
  stop(): void;
  isSpinning: boolean;
}

export interface CLIProgress {
  start(total: number, initialValue?: number): void;
  update(current: number, payload?: any): void;
  increment(step?: number, payload?: any): void;
  stop(): void;
}

export interface CLIInteractive {
  confirm(message: string, defaultValue?: boolean): Promise<boolean>;
  input(message: string, defaultValue?: string): Promise<string>;
  password(message: string): Promise<string>;
  select<T = string>(message: string, choices: T[] | SelectChoice<T>[]): Promise<T>;
  multiSelect<T = string>(message: string, choices: T[] | SelectChoice<T>[]): Promise<T[]>;
  autocomplete(message: string, source: (input: string) => Promise<string[]>): Promise<string>;
}

export interface SelectChoice<T = string> {
  title: string;
  value: T;
  description?: string;
  disabled?: boolean;
}

export interface CompilerCLIOptions {
  source: string;
  output?: string;
  optimize?: boolean;
  optimize_runs?: number;
  evm_version?: string;
  solc_version?: string;
  include_paths?: string[];
  remappings?: string[];
  metadata_hash?: 'none' | 'ipfs' | 'bzzr1';
  metadata_literal?: boolean;
  libraries?: string[];
  combined_json?: string[];
  pretty_json?: boolean;
  gas?: boolean;
  ast?: boolean;
  asm?: boolean;
  bin?: boolean;
  bin_runtime?: boolean;
  abi?: boolean;
  storage_layout?: boolean;
  debug_info?: boolean;
  stop_after?: string;
  base_path?: string;
  allow_paths?: string[];
}

export interface DeployerCLIOptions {
  network: string;
  contract?: string;
  args?: string[];
  gas_limit?: string;
  gas_price?: string;
  value?: string;
  private_key?: string;
  mnemonic?: string;
  keystore?: string;
  password?: string;
  confirm?: boolean;
  dry_run?: boolean;
  verify?: boolean;
  wait_confirmations?: number;
  timeout?: number;
  json?: boolean;
  silent?: boolean;
}

export interface TestCLIOptions {
  match_contract?: string;
  no_match_contract?: string;
  match_test?: string;
  no_match_test?: string;
  match_path?: string;
  no_match_path?: string;
  fork_url?: string;
  fork_block?: number;
  verbosity?: number;
  gas_report?: boolean;
  coverage?: boolean;
  json?: boolean;
  fail_fast?: boolean;
  watch?: boolean;
  debug?: boolean;
  trace?: boolean;
}

export interface InitCLIOptions {
  name?: string;
  template?: string;
  version?: string;
  author?: string;
  license?: string;
  description?: string;
  git?: boolean;
  install?: boolean;
  package_manager?: 'npm' | 'yarn' | 'pnpm';
  force?: boolean;
  interactive?: boolean;
}

export interface BuildCLIOptions {
  root?: string;
  src?: string;
  out?: string;
  libs?: string[];
  remappings?: string[];
  solc_version?: string;
  evm_version?: string;
  optimizer?: boolean;
  optimizer_runs?: number;
  via_ir?: boolean;
  watch?: boolean;
  force?: boolean;
  names?: boolean;
  sizes?: boolean;
  json?: boolean;
}

export interface CLIResult<T = any> {
  success: boolean;
  data?: T;
  error?: {
    message: string;
    code?: number;
    stack?: string;
  };
  warnings?: string[];
  duration: number;
  metadata?: {
    [key: string]: any;
  };
}

export interface GlobalCLIOptions {
  config?: string;
  verbose?: boolean;
  quiet?: boolean;
  color?: boolean;
  json?: boolean;
  help?: boolean;
  version?: boolean;
}

export interface CLIConfig {
  defaults: {
    [command: string]: { [option: string]: any };
  };
  profiles: {
    [profile: string]: { [option: string]: any };
  };
  plugins: string[];
  aliases: {
    [alias: string]: string;
  };
}

export interface PluginCLI {
  name: string;
  version: string;
  commands: CLICommand[];
  hooks: {
    [hookName: string]: (context: CLIContext) => Promise<void>;
  };
}

export interface HookContext extends CLIContext {
  command: string;
  args: CLIArgs;
  result?: any;
  error?: Error;
}

export interface CLITable {
  headers: string[];
  rows: (string | number)[][];
  options?: {
    padding?: number;
    alignment?: ('left' | 'center' | 'right')[];
    colors?: boolean;
    borders?: boolean;
  };
}

export interface CLIFormatter {
  table(data: CLITable): string;
  json(data: any, pretty?: boolean): string;
  yaml(data: any): string;
  csv(data: any[]): string;
  tree(data: any, options?: TreeOptions): string;
}

export interface TreeOptions {
  unicode?: boolean;
  colors?: boolean;
  maxDepth?: number;
  showValues?: boolean;
}