import {
  ProjectTemplate,
  TemplateFile,
  TemplateContext,
  TemplateEngine as ITemplateEngine,
  TemplateHelper,
  ScaffoldOptions,
  ScaffoldResult,
  TemplateRegistry as ITemplateRegistry,
  TemplateGenerator as ITemplateGenerator,
  BasicTemplateOptions,
  ERC20TemplateOptions,
  ERC721TemplateOptions,
  CustomTemplateOptions,
  PostInstallAction,
  MIN_SUPPORTED_SOLIDITY_VERSION,
  SUPPORTED_SOLIDITY_RANGE,
  isSupportedSolidityVersion,
} from '@neo-devpack-solidity/types';
import fs from 'fs-extra';
import * as path from 'path';
import { EventEmitter } from 'events';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export class TemplateEngine extends EventEmitter implements ITemplateEngine {
  private helpers: Map<string, TemplateHelper> = new Map();
  private partials: Map<string, string> = new Map();

  constructor() {
    super();
    this.registerDefaultHelpers();
  }

  render(template: string, context: TemplateContext): string {
    let result = template;

    // Replace variables
    result = this.replaceVariables(result, context);
    
    // Process helpers
    result = this.processHelpers(result, context);
    
    // Process partials
    result = this.processPartials(result, context);

    return result;
  }

  renderFile(filePath: string, context: TemplateContext): string {
    const template = fs.readFileSync(filePath, 'utf8');
    return this.render(template, context);
  }

  registerHelper(name: string, helper: TemplateHelper): void {
    this.helpers.set(name, helper);
  }

  registerPartial(name: string, partial: string): void {
    this.partials.set(name, partial);
  }

  private replaceVariables(template: string, context: TemplateContext): string {
    return template.replace(/\{\{\s*([@\w]+)\s*\}\}/g, (match, key) => {
      return context[key] !== undefined ? String(context[key]) : match;
    });
  }

  private processHelpers(template: string, context: TemplateContext): string {
    return template.replace(/\{\{#(\w+)\s*([^}]*)\}\}(.*?)\{\{\/\1\}\}/gs, (match, helperName, args, content) => {
      const helper = this.helpers.get(helperName);
      if (helper) {
        return helper(content.trim(), { args: args.trim(), context });
      }
      return match;
    });
  }

  private processPartials(template: string, context: TemplateContext): string {
    return template.replace(/\{\{>\s*(\w+)\s*\}\}/g, (match, partialName) => {
      const partial = this.partials.get(partialName);
      if (partial) {
        return this.render(partial, context);
      }
      return match;
    });
  }

  private registerDefaultHelpers(): void {
    // Conditional helper
    this.registerHelper('if', (content: string, options: any) => {
      const condition = options.args;
      const context = options.context;
      
      // Simple condition evaluation
      if (context[condition]) {
        return content;
      }
      return '';
    });

    // Loop helper
    this.registerHelper('each', (content: string, options: any) => {
      const arrayName = options.args;
      const context = options.context;
      const array = context[arrayName];
      
      if (Array.isArray(array)) {
        return array.map((item, index) => {
          const itemContext = { ...context, this: item, "@index": index };
          return this.render(content, itemContext);
        }).join('');
      }
      return '';
    });

    // Uppercase helper
    this.registerHelper('upper', (content: string) => {
      return content.toUpperCase();
    });

    // Lowercase helper
    this.registerHelper('lower', (content: string) => {
      return content.toLowerCase();
    });

    // Capitalize helper
    this.registerHelper('capitalize', (content: string) => {
      return content.charAt(0).toUpperCase() + content.slice(1);
    });

    // Date helper
    this.registerHelper('date', () => {
      return new Date().toISOString().split('T')[0];
    });

    // Year helper
    this.registerHelper('year', () => {
      return new Date().getFullYear().toString();
    });
  }
}

export class ProjectScaffolder extends EventEmitter {
  private templateEngine: TemplateEngine;
  private registry: ITemplateRegistry;

  constructor() {
    super();
    this.templateEngine = new TemplateEngine();
    this.registry = new TemplateRegistry();
    this.loadBuiltinTemplates();
  }

  async scaffold(options: ScaffoldOptions): Promise<ScaffoldResult> {
    const startTime = Date.now();
    this.emit('scaffoldStarted', options);

    try {
      // Get template
      const template = this.registry.get(options.template);
      if (!template) {
        throw new Error(`Template '${options.template}' not found`);
      }

      // Prepare context
      const context = await this.prepareContext(options, template);

      // Determine project directory
      const projectPath = options.directory || path.join(process.cwd(), options.name);
      
      // Check if directory exists and handle force option
      if (await fs.pathExists(projectPath) && !options.force) {
        throw new Error(`Directory ${projectPath} already exists. Use --force to overwrite.`);
      }

      if (options.dryRun) {
        return this.simulateScaffold(template, context, projectPath);
      }

      // Create project structure
      const result = await this.createProject(template, context, projectPath);
      
      result.duration = Date.now() - startTime;
      this.emit('scaffoldCompleted', result);
      
      return result;
    } catch (error) {
      const result: ScaffoldResult = {
        success: false,
        projectPath: '',
        filesCreated: [],
        dependencies: [],
        devDependencies: [],
        postInstallActions: [],
        errors: [String(error)],
        warnings: [],
        duration: Date.now() - startTime
      };

      this.emit('scaffoldFailed', { error, result });
      return result;
    }
  }

  private async prepareContext(options: ScaffoldOptions, template: ProjectTemplate): Promise<TemplateContext> {
    const context: TemplateContext = {
      projectName: options.name,
      author: options.context?.author || 'Unknown',
      description: options.context?.description || `A new ${template.name} project`,
      license: options.context?.license || template.license || 'MIT',
      version: options.context?.version || '1.0.0',
      packageManager: options.packageManager || 'npm',
      gitInit: options.gitInit !== false,
      installDependencies: options.install !== false,
      ...options.context
    };

    // Interactive mode
    if (options.interactive) {
      await this.promptForContext(context, template);
    }

    return context;
  }

  private async promptForContext(context: TemplateContext, template: ProjectTemplate): Promise<void> {
    // This would use inquirer to prompt for missing values
    // Simplified implementation for now
    console.log(`Creating ${template.name} project: ${context.projectName}`);
  }

  private async simulateScaffold(
    template: ProjectTemplate,
    context: TemplateContext,
    projectPath: string
  ): Promise<ScaffoldResult> {
    const filesCreated: string[] = [];

    for (const file of template.files) {
      if (!this.shouldIncludeFile(file, context)) {
        continue;
      }

      const targetPath = path.join(projectPath, this.templateEngine.render(file.path, context));
      filesCreated.push(targetPath);
    }

    return {
      success: true,
      projectPath,
      filesCreated,
      dependencies: Object.keys(template.dependencies),
      devDependencies: Object.keys(template.devDependencies),
      postInstallActions: template.postInstall || [],
      errors: [],
      warnings: ['This was a dry run - no files were actually created'],
      duration: 0
    };
  }

  private async createProject(
    template: ProjectTemplate,
    context: TemplateContext,
    projectPath: string
  ): Promise<ScaffoldResult> {
    const filesCreated: string[] = [];
    const errors: string[] = [];
    const warnings: string[] = [];

    // Ensure project directory exists
    await fs.ensureDir(projectPath);

    // Create files
    for (const file of template.files) {
      try {
        if (!this.shouldIncludeFile(file, context)) {
          continue;
        }

        const targetPath = path.join(projectPath, this.templateEngine.render(file.path, context));
        await fs.ensureDir(path.dirname(targetPath));

        let content: string;
        if (file.template) {
          content = this.templateEngine.render(file.content as string, context);
        } else {
          content = file.content as string;
        }

        await fs.writeFile(targetPath, content, { mode: file.permissions });
        filesCreated.push(targetPath);
      } catch (error) {
        errors.push(`Failed to create file ${file.path}: ${error}`);
      }
    }

    // Create package.json
    const packageJson = await this.createPackageJson(template, context, projectPath);
    if (packageJson) {
      filesCreated.push(packageJson);
    }

    // Execute post-install actions
    const postInstallActions = template.postInstall || [];
    for (const action of postInstallActions) {
      try {
        await this.executePostInstallAction(action, projectPath, context);
      } catch (error) {
        warnings.push(`Post-install action failed: ${error}`);
      }
    }

    return {
      success: errors.length === 0,
      projectPath,
      filesCreated,
      dependencies: Object.keys(template.dependencies),
      devDependencies: Object.keys(template.devDependencies),
      postInstallActions,
      errors,
      warnings,
      duration: 0
    };
  }

  private shouldIncludeFile(file: TemplateFile, context: TemplateContext): boolean {
    if (!file.conditional) {
      return true;
    }

    // Simple condition evaluation
    return Boolean(context[file.conditional]);
  }

  private async createPackageJson(
    template: ProjectTemplate,
    context: TemplateContext,
    projectPath: string
  ): Promise<string | null> {
    const packageJson = {
      name: context.projectName,
      version: context.version,
      description: context.description,
      author: context.author,
      license: context.license,
      scripts: { ...template.scripts },
      dependencies: { ...template.dependencies },
      devDependencies: { ...template.devDependencies },
      engines: template.requirements
    };

    const packageJsonPath = path.join(projectPath, 'package.json');
    await fs.writeFile(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);
    
    return packageJsonPath;
  }

  private async executePostInstallAction(
    action: PostInstallAction,
    projectPath: string,
    context: TemplateContext
  ): Promise<void> {
    if (action.condition && !context[action.condition]) {
      return;
    }

    switch (action.type) {
      case 'command':
        if (action.command) {
          await execAsync(action.command, { cwd: projectPath });
        }
        break;

      case 'git':
        if (context.gitInit) {
          await execAsync('git init', { cwd: projectPath });
          await execAsync('git add .', { cwd: projectPath });
          await execAsync('git commit -m "Initial commit"', { cwd: projectPath });
        }
        break;

      case 'npm':
        if (context.installDependencies && context.packageManager === 'npm') {
          await execAsync('npm install', { cwd: projectPath });
        }
        break;

      case 'file':
        // Handle file operations
        break;
    }
  }

  private loadBuiltinTemplates(): void {
    const generator = new TemplateGenerator();
    
    // Load built-in templates
    const templates = [
      generator.generateBasic({
        name: 'basic',
        author: 'Neo DevPack for Solidity Team',
        description: 'Basic Neo DevPack for Solidity project',
        license: 'MIT',
        solcVersion: MIN_SUPPORTED_SOLIDITY_VERSION,
        includeTests: true,
        includeDocs: false,
        framework: 'hardhat'
      }),
      generator.generateERC20({
        name: 'erc20',
        author: 'Neo DevPack for Solidity Team',
        description: 'ERC20 Token project',
        license: 'MIT',
        solcVersion: MIN_SUPPORTED_SOLIDITY_VERSION,
        includeTests: true,
        includeDocs: true,
        framework: 'hardhat',
        tokenName: '{{tokenName}}',
        tokenSymbol: '{{tokenSymbol}}',
        decimals: 18,
        totalSupply: '1000000',
        mintable: true,
        burnable: false,
        pausable: false,
        ownable: true,
        accessControl: false
      })
    ];

    templates.forEach(template => this.registry.register(template));
  }
}

export class TemplateRegistry implements ITemplateRegistry {
  templates: Map<string, ProjectTemplate> = new Map();

  register(template: ProjectTemplate): void {
    this.templates.set(template.name, template);
  }

  get(name: string): ProjectTemplate | undefined {
    return this.templates.get(name);
  }

  list(): ProjectTemplate[] {
    return Array.from(this.templates.values());
  }

  search(query: string): ProjectTemplate[] {
    const results: ProjectTemplate[] = [];
    const lowerQuery = query.toLowerCase();

    for (const template of this.templates.values()) {
      if (
        template.name.toLowerCase().includes(lowerQuery) ||
        template.description.toLowerCase().includes(lowerQuery) ||
        template.tags.some(tag => tag.toLowerCase().includes(lowerQuery))
      ) {
        results.push(template);
      }
    }

    return results;
  }

  validate(template: ProjectTemplate): any {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Required fields
    if (!template.name) errors.push('Template name is required');
    if (!template.version) errors.push('Template version is required');
    if (!template.files || template.files.length === 0) errors.push('Template must have at least one file');

    // File validation
    for (const file of template.files || []) {
      if (!file.path) errors.push('File path is required');
      if (!file.content) errors.push(`File content is required for ${file.path}`);
    }

    // Dependencies validation
    if (template.dependencies) {
      for (const [name, version] of Object.entries(template.dependencies)) {
        if (!version || typeof version !== 'string') {
          warnings.push(`Invalid version for dependency ${name}`);
        }
      }
    }

    return {
      valid: errors.length === 0,
      errors: errors.map(msg => ({ type: 'missing_field' as const, message: msg })),
      warnings: warnings.map(msg => ({ type: 'potential_conflict' as const, message: msg }))
    };
  }
}

export class TemplateGenerator implements ITemplateGenerator {
  private validateSolcVersion(version?: string): void {
    const effective = version || MIN_SUPPORTED_SOLIDITY_VERSION;
    if (!isSupportedSolidityVersion(effective)) {
      throw new Error(
        `Unsupported Solidity version "${effective}". Supported range is ${SUPPORTED_SOLIDITY_RANGE}.`
      );
    }
  }

  generateBasic(options: BasicTemplateOptions): ProjectTemplate {
    this.validateSolcVersion(options.solcVersion);
    const files: TemplateFile[] = [
      {
        path: 'contracts/{{contractName}}.sol',
        content: this.getBasicContractTemplate(),
        template: true
      },
      {
        path: 'hardhat.config.js',
        content: this.getHardhatConfigTemplate(),
        template: true,
        conditional: options.framework === 'hardhat' ? undefined : 'useHardhat'
      },
      {
        path: 'foundry.toml',
        content: this.getFoundryConfigTemplate(),
        template: true,
        conditional: options.framework === 'foundry' ? undefined : 'useFoundry'
      }
    ];

    if (options.includeTests) {
      files.push({
        path: 'test/{{contractName}}.test.js',
        content: this.getBasicTestTemplate(),
        template: true
      });
    }

    if (options.includeDocs) {
      files.push(
        {
          path: 'README.md',
          content: this.getReadmeTemplate(),
          template: true
        },
        {
          path: 'docs/{{contractName}}.md',
          content: this.getDocTemplate(),
          template: true
        }
      );
    }

    return {
      name: options.name,
      description: options.description,
      version: '1.0.0',
      author: options.author,
      license: options.license,
      tags: ['basic', 'starter'],
      requirements: {
        node: '>=18.0.0'
      },
      files,
      dependencies: {},
      devDependencies: {
        'hardhat': '^2.28.6',
        '@neo-devpack-solidity/hardhat-solc-neo': '^0.14.0',
        '@neo-devpack-solidity/hardhat-neo-deployer': '^0.14.0',
        'chai': '^4.5.0'
      },
      scripts: {
        'compile': 'npx hardhat neo-compile',
        'test': 'npx hardhat test --no-compile',
        'deploy': 'npx hardhat neo-deploy',
        'verify': 'npx hardhat neo-verify'
      },
      postInstall: [
        { type: 'git', condition: 'gitInit' },
        { type: 'npm', condition: 'installDependencies' }
      ],
      configuration: {
        hardhat: options.framework === 'hardhat' ? {} : undefined,
        foundry: options.framework === 'foundry' ? {} : undefined
      }
    };
  }

  generateERC20(options: ERC20TemplateOptions): ProjectTemplate {
    // Implementation for ERC20 template
    const basic = this.generateBasic(options);
    
    basic.files.push({
      path: 'contracts/{{tokenName}}.sol',
      content: this.getERC20ContractTemplate(),
      template: true
    });

    basic.tags.push('erc20', 'token');
    
    return basic;
  }

  generateERC721(options: ERC721TemplateOptions): ProjectTemplate {
    // Implementation for ERC721 template
    const basic = this.generateBasic(options);
    
    basic.files.push({
      path: 'contracts/{{tokenName}}.sol',
      content: this.getERC721ContractTemplate(),
      template: true
    });

    basic.tags.push('erc721', 'nft');
    
    return basic;
  }

  generateCustom(options: CustomTemplateOptions): ProjectTemplate {
    const basic = this.generateBasic(options);
    
    // Add custom contracts
    options.contracts.forEach(contract => {
      basic.files.push({
        path: `contracts/${contract.path}`,
        content: contract.content,
        template: true
      });
    });

    // Add custom tests
    options.tests.forEach(test => {
      basic.files.push({
        path: `test/${test.path}`,
        content: test.content,
        template: true
      });
    });

    // Add custom scripts
    options.scripts.forEach(script => {
      basic.files.push({
        path: `scripts/${script.path}`,
        content: script.content,
        template: true
      });
    });

    return basic;
  }

  // Template Content Methods
  private getBasicContractTemplate(): string {
    return `// SPDX-License-Identifier: {{license}}
pragma solidity ^0.8.19;

/**
 * @title {{contractName}}
 * @dev {{description}}
 */
contract {{contractName}} {
    address public owner;
    
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not the contract owner");
        _;
    }
    
    constructor() {
        owner = msg.sender;
        emit OwnershipTransferred(address(0), owner);
    }
    
    function transferOwnership(address newOwner) public onlyOwner {
        require(newOwner != address(0), "New owner is the zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }
}`;
  }

  private getERC20ContractTemplate(): string {
    return `// SPDX-License-Identifier: {{license}}
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
{{#if ownable}}import "@openzeppelin/contracts/access/Ownable.sol";{{/if}}

contract {{tokenName}} is ERC20{{#if ownable}}, Ownable{{/if}} {
    constructor() ERC20("{{tokenName}}", "{{tokenSymbol}}") {
        _mint(msg.sender, {{totalSupply}} * 10 ** decimals());
    }
    
    {{#if mintable}}
    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
    {{/if}}
}`;
  }

  private getERC721ContractTemplate(): string {
    return `// SPDX-License-Identifier: {{license}}
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract {{tokenName}} is ERC721, Ownable {
    uint256 private _tokenIdCounter;
    
    constructor() ERC721("{{tokenName}}", "{{tokenSymbol}}") {}
    
    function mint(address to) public onlyOwner {
        _tokenIdCounter++;
        _mint(to, _tokenIdCounter);
    }
}`;
  }

  private getHardhatConfigTemplate(): string {
    return `require("@neo-devpack-solidity/hardhat-solc-neo");
require("@neo-devpack-solidity/hardhat-neo-deployer");

module.exports = {
  solidity: {
    version: "{{solcVersion}}"
  },
  neoSolc: {
    solidity: {
      version: "{{solcVersion}}",
      settings: {
        optimizer: { enabled: true, runs: 200 },
        neo: { callt: true }
      }
    }
  },
  neoNetworks: {
    testnet: {
      rpcUrls: ["{{neoRpcUrl}}"],
      magic: 894710606,
      addressVersion: 0x35,
      accounts: [process.env.NEO_WIF || ""]
    }
  }
};`;
  }

  private getFoundryConfigTemplate(): string {
    return `[profile.default]
src = "src"
out = "out"
libs = ["lib"]
solc-version = "{{solcVersion}}"
optimizer = true
optimizer-runs = 200

[rpc_endpoints]
neo = "{{neoRpcUrl}}"`;
  }

  private getBasicTestTemplate(): string {
    return `const { expect } = require("chai");
const hre = require("hardhat");

describe("{{contractName}} build", function () {
  it("emits Neo build artifacts", async function () {
    await hre.run("neo-compile", { force: true, quiet: true });

    const artifact = await hre.neoSolc.artifacts.getBuildArtifact("{{contractName}}");

    expect(artifact, "missing build artifact").to.not.equal(null);
    expect(artifact.contract.neo.manifest.name).to.equal("{{contractName}}");
    expect(artifact.contract.neo.manifest.abi.methods).to.not.be.empty;
  });
});`;
  }

  private getReadmeTemplate(): string {
    return `# {{projectName}}

{{description}}

## Installation

\`\`\`bash
{{packageManager}} install
\`\`\`

## Usage

### Compile contracts
\`\`\`bash
{{packageManager}} run compile
\`\`\`

### Run tests
\`\`\`bash
{{packageManager}} run test
\`\`\`

### Deploy
\`\`\`bash
{{packageManager}} run deploy
\`\`\`

## License

{{license}}`;
  }

  private getDocTemplate(): string {
    return `# {{contractName}}

{{description}}

## Overview

This contract provides basic functionality for...

## Functions

### Constructor

Initializes the contract with...

### Public Functions

- \`transferOwnership(address newOwner)\`: Transfers ownership to a new address

## Events

- \`OwnershipTransferred\`: Emitted when ownership is transferred`;
  }
}
