export interface ProjectTemplate {
  name: string;
  description: string;
  version: string;
  author: string;
  license: string;
  tags: string[];
  requirements: {
    node?: string;
    npm?: string;
    solc?: string;
  };
  files: TemplateFile[];
  dependencies: {
    [packageName: string]: string;
  };
  devDependencies: {
    [packageName: string]: string;
  };
  scripts: {
    [scriptName: string]: string;
  };
  postInstall?: PostInstallAction[];
  configuration: TemplateConfiguration;
}

export interface TemplateFile {
  path: string;
  content: string | Buffer;
  permissions?: string;
  template?: boolean;
  conditional?: string;
}

export interface TemplateConfiguration {
  hardhat?: any;
  foundry?: any;
  networks?: NetworkTemplateConfig[];
  compiler?: CompilerTemplateConfig;
}

export interface NetworkTemplateConfig {
  name: string;
  chainId: number;
  rpcUrl: string;
  blockExplorer?: string;
  gasPrice?: string;
  gasLimit?: string;
  accounts?: AccountConfig[];
}

export interface AccountConfig {
  type: 'private_key' | 'mnemonic' | 'keystore';
  value?: string;
  path?: string;
  count?: number;
}

export interface CompilerTemplateConfig {
  version: string;
  settings: {
    optimizer: {
      enabled: boolean;
      runs: number;
    };
    evmVersion: string;
    outputSelection: any;
  };
}

export interface PostInstallAction {
  type: 'command' | 'git' | 'npm' | 'file';
  command?: string;
  args?: string[];
  condition?: string;
}

export interface TemplateContext {
  projectName: string;
  author: string;
  description: string;
  license: string;
  version: string;
  packageManager: 'npm' | 'yarn' | 'pnpm';
  gitInit: boolean;
  installDependencies: boolean;
  [key: string]: any;
}

export interface TemplateEngine {
  render(template: string, context: TemplateContext): string;
  renderFile(filePath: string, context: TemplateContext): string;
  registerHelper(name: string, helper: TemplateHelper): void;
  registerPartial(name: string, partial: string): void;
}

export type TemplateHelper = (context: any, options?: any) => string;

export interface ScaffoldOptions {
  template: string;
  name: string;
  directory?: string;
  context?: Partial<TemplateContext>;
  force?: boolean;
  interactive?: boolean;
  dryRun?: boolean;
  gitInit?: boolean;
  install?: boolean;
  packageManager?: 'npm' | 'yarn' | 'pnpm';
}

export interface ScaffoldResult {
  success: boolean;
  projectPath: string;
  filesCreated: string[];
  dependencies: string[];
  devDependencies: string[];
  postInstallActions: PostInstallAction[];
  errors: string[];
  warnings: string[];
  duration: number;
}

export interface TemplateRegistry {
  templates: Map<string, ProjectTemplate>;
  
  register(template: ProjectTemplate): void;
  get(name: string): ProjectTemplate | undefined;
  list(): ProjectTemplate[];
  search(query: string): ProjectTemplate[];
  validate(template: ProjectTemplate): TemplateValidationResult;
}

export interface TemplateValidationResult {
  valid: boolean;
  errors: TemplateValidationError[];
  warnings: TemplateValidationWarning[];
}

export interface TemplateValidationError {
  type: 'missing_field' | 'invalid_file' | 'dependency_conflict' | 'invalid_script';
  message: string;
  path?: string;
}

export interface TemplateValidationWarning {
  type: 'deprecated_dependency' | 'large_file' | 'potential_conflict';
  message: string;
  path?: string;
}

export interface BuiltinTemplates {
  'basic': ProjectTemplate;
  'erc20': ProjectTemplate;
  'erc721': ProjectTemplate;
  'defi': ProjectTemplate;
  'dao': ProjectTemplate;
  'multisig': ProjectTemplate;
  'proxy': ProjectTemplate;
  'library': ProjectTemplate;
  'hardhat-basic': ProjectTemplate;
  'foundry-basic': ProjectTemplate;
  'mixed': ProjectTemplate;
}

export interface TemplateGenerator {
  generateBasic(options: BasicTemplateOptions): ProjectTemplate;
  generateERC20(options: ERC20TemplateOptions): ProjectTemplate;
  generateERC721(options: ERC721TemplateOptions): ProjectTemplate;
  generateCustom(options: CustomTemplateOptions): ProjectTemplate;
}

export interface BasicTemplateOptions {
  name: string;
  author: string;
  description: string;
  license: string;
  solcVersion: string;
  includeTests: boolean;
  includeDocs: boolean;
  framework: 'hardhat' | 'foundry' | 'mixed';
}

export interface ERC20TemplateOptions extends BasicTemplateOptions {
  tokenName: string;
  tokenSymbol: string;
  decimals: number;
  totalSupply: string;
  mintable: boolean;
  burnable: boolean;
  pausable: boolean;
  ownable: boolean;
  accessControl: boolean;
}

export interface ERC721TemplateOptions extends BasicTemplateOptions {
  tokenName: string;
  tokenSymbol: string;
  baseURI: string;
  enumerable: boolean;
  metadata: boolean;
  pausable: boolean;
  ownable: boolean;
  accessControl: boolean;
  royalties: boolean;
}

export interface CustomTemplateOptions extends BasicTemplateOptions {
  contracts: ContractTemplate[];
  tests: TestTemplate[];
  scripts: ScriptTemplate[];
  documentation: DocumentationTemplate[];
}

export interface ContractTemplate {
  name: string;
  path: string;
  content: string;
  dependencies: string[];
}

export interface TestTemplate {
  name: string;
  path: string;
  content: string;
  framework: 'hardhat' | 'foundry';
}

export interface ScriptTemplate {
  name: string;
  path: string;
  content: string;
  type: 'deployment' | 'interaction' | 'utility';
}

export interface DocumentationTemplate {
  name: string;
  path: string;
  content: string;
  format: 'markdown' | 'rst' | 'html';
}

export interface TemplateUpgrade {
  from: string;
  to: string;
  migrations: TemplateMigration[];
}

export interface TemplateMigration {
  type: 'file_rename' | 'file_content' | 'dependency_update' | 'script_update';
  description: string;
  execute: (projectPath: string) => Promise<void>;
}

export interface TemplateMetadata {
  name: string;
  version: string;
  createdAt: Date;
  updatedAt: Date;
  downloadCount: number;
  rating: number;
  tags: string[];
  screenshots?: string[];
  readme?: string;
  changelog?: string[];
}