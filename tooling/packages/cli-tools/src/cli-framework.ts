import {
  CLICommand,
  CLIArgs,
  CLIContext,
  CLILogger,
  CLISpinner,
  CLIProgress,
  CLIInteractive,
  CLIResult,
  CLIConfig,
  CLIFormatter,
  CLITable
} from '@neo-devpack-solidity/types';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import chalk from 'chalk';
import inquirer from 'inquirer';
import ora, { Ora } from 'ora';
import * as fs from 'fs-extra';
import * as path from 'path';
import * as os from 'os';
import { EventEmitter } from 'events';

export class NeoDevpackSolidityCLI extends EventEmitter {
  private commands: Map<string, CLICommand> = new Map();
  private config: CLIConfig;
  private context: CLIContext;
  private yargs: any;

  constructor(config: Partial<CLIConfig> = {}) {
    super();
    
    this.config = {
      defaults: {},
      profiles: {},
      plugins: [],
      aliases: {},
      ...config
    };

    this.context = {
      config: this.config,
      logger: new ConsoleLogger(),
      spinner: new SpinnerManager(),
      progress: new ProgressManager(),
      interactive: new InteractiveManager()
    };

    this.yargs = yargs(hideBin(process.argv));
    this.setupGlobalOptions();
  }

  // Command Registration
  register(command: CLICommand): void {
    this.commands.set(command.name, command);
    
    // Register aliases
    if (command.aliases) {
      command.aliases.forEach(alias => {
        this.commands.set(alias, command);
      });
    }

    this.emit('commandRegistered', command);
  }

  registerMultiple(commands: CLICommand[]): void {
    commands.forEach(command => this.register(command));
  }

  // Command Execution
  async execute(argv: string[] = process.argv): Promise<CLIResult> {
    try {
      const startTime = Date.now();
      this.emit('executionStarted', { argv });

      // Build yargs configuration
      this.buildYargsConfig();

      // Parse arguments
      const args = await this.yargs.parse(argv.slice(2));
      
      // Execute command
      const result = await this.executeCommand(args);
      
      result.duration = Date.now() - startTime;
      this.emit('executionCompleted', result);
      
      return result;
    } catch (error) {
      const result: CLIResult = {
        success: false,
        error: {
          message: String(error),
          code: error instanceof Error && 'code' in error ? (error as any).code : -1,
          stack: error instanceof Error ? error.stack : undefined
        },
        duration: 0
      };

      this.emit('executionFailed', { error, result });
      return result;
    }
  }

  // Configuration Management
  loadConfig(configPath?: string): void {
    const configFile = configPath || this.findConfigFile();
    
    if (configFile && fs.existsSync(configFile)) {
      try {
        const fileConfig = fs.readJsonSync(configFile);
        this.config = { ...this.config, ...fileConfig };
        this.context.config = this.config;
        this.emit('configLoaded', { configFile, config: this.config });
      } catch (error) {
        this.context.logger.warn(`Failed to load config from ${configFile}: ${error}`);
      }
    }
  }

  saveConfig(configPath?: string): void {
    const configFile = configPath || path.join(process.cwd(), '.neodevpacksolidity.json');
    
    try {
      fs.writeJsonSync(configFile, this.config, { spaces: 2 });
      this.emit('configSaved', { configFile, config: this.config });
    } catch (error) {
      throw new Error(`Failed to save config to ${configFile}: ${error}`);
    }
  }

  // Help and Documentation
  showHelp(commandName?: string): void {
    if (commandName && this.commands.has(commandName)) {
      this.showCommandHelp(this.commands.get(commandName)!);
    } else {
      this.showGlobalHelp();
    }
  }

  // Private Implementation
  private setupGlobalOptions(): void {
    this.yargs
      .option('config', {
        alias: 'c',
        type: 'string',
        description: 'Path to configuration file'
      })
      .option('verbose', {
        alias: 'v',
        type: 'boolean',
        description: 'Enable verbose output'
      })
      .option('quiet', {
        alias: 'q',
        type: 'boolean',
        description: 'Suppress output'
      })
      .option('json', {
        type: 'boolean',
        description: 'Output in JSON format'
      })
      .option('color', {
        type: 'boolean',
        default: true,
        description: 'Enable colored output'
      })
      .help()
      .version();
  }

  private buildYargsConfig(): void {
    for (const [name, command] of this.commands.entries()) {
      if (name === command.name) { // Only register once, not for aliases
        this.yargs.command(
          this.buildCommandSignature(command),
          command.description,
          (yargs: any) => this.buildCommandOptions(yargs, command),
          () => {}
        );
      }
    }
  }

  private buildCommandSignature(command: CLICommand): string {
    const positionals = command.options
      .filter(opt => !opt.name.startsWith('-'))
      .map(opt => opt.required ? `<${opt.name}>` : `[${opt.name}]`)
      .join(' ');

    return positionals ? `${command.name} ${positionals}` : command.name;
  }

  private buildCommandOptions(yargs: any, command: CLICommand): any {
    for (const option of command.options) {
      const yargsOption: any = {
        type: option.type === 'array' ? 'array' : option.type,
        description: option.description,
        required: option.required,
        default: option.default,
        choices: option.choices,
        alias: option.alias,
        hidden: option.hidden
      };

      yargs.option(option.name, yargsOption);
    }

    // Add examples
    if (command.examples) {
      command.examples.forEach(example => {
        yargs.example(example.command, example.description);
      });
    }

    return yargs;
  }

  private async executeCommand(args: CLIArgs): Promise<CLIResult> {
    const commandName = args._[0];
    
    if (!commandName) {
      this.showGlobalHelp();
      return { success: true, duration: 0 };
    }

    const command = this.commands.get(commandName);
    if (!command) {
      throw new Error(`Unknown command: ${commandName}`);
    }

    await command.action(args);
    return { success: true, duration: 0 };
  }

  private findConfigFile(): string | null {
    const candidates = [
      path.join(process.cwd(), '.neodevpacksolidity.json'),
      path.join(process.cwd(), 'neodevpacksolidity.config.json'),
      path.join(os.homedir(), '.neodevpacksolidity.json')
    ];

    return candidates.find(file => fs.existsSync(file)) || null;
  }

  private showGlobalHelp(): void {
    console.log(chalk.blue.bold('Neo DevPack for Solidity CLI'));
    console.log(chalk.gray('Comprehensive tooling for Neo DevPack for Solidity development'));
    console.log('');
    console.log(chalk.yellow('Available Commands:'));
    
    const commandsByCategory: { [category: string]: CLICommand[] } = {};
    
    for (const [name, command] of this.commands.entries()) {
      if (name === command.name) { // Only show once, not for aliases
        const category = this.getCommandCategory(command);
        if (!commandsByCategory[category]) {
          commandsByCategory[category] = [];
        }
        commandsByCategory[category].push(command);
      }
    }

    for (const [category, commands] of Object.entries(commandsByCategory)) {
      console.log(chalk.cyan(`\n${category}:`));
      commands.forEach(command => {
        const aliases = command.aliases ? ` (${command.aliases.join(', ')})` : '';
        console.log(`  ${command.name.padEnd(20)} ${command.description}${chalk.gray(aliases)}`);
      });
    }

    console.log(chalk.yellow('\nGlobal Options:'));
    console.log('  --config, -c <path>   Configuration file path');
    console.log('  --verbose, -v         Enable verbose output');
    console.log('  --quiet, -q           Suppress output');
    console.log('  --json                Output in JSON format');
    console.log('  --color               Enable colored output');
    console.log('  --help                Show help');
    console.log('  --version             Show version');
  }

  private showCommandHelp(command: CLICommand): void {
    console.log(chalk.blue.bold(`${command.name}`));
    console.log(command.description);
    console.log('');

    if (command.aliases && command.aliases.length > 0) {
      console.log(chalk.yellow('Aliases:'), command.aliases.join(', '));
      console.log('');
    }

    if (command.options.length > 0) {
      console.log(chalk.yellow('Options:'));
      command.options.forEach(option => {
        const flags = [
          `--${option.name}`,
          option.alias ? `-${option.alias}` : undefined
        ].filter(Boolean).join(', ');
        
        const typeInfo = option.type === 'array' ? ' <array>' : 
                        option.type === 'string' ? ' <string>' :
                        option.type === 'number' ? ' <number>' : '';
        
        const required = option.required ? chalk.red(' (required)') : '';
        const defaultValue = option.default !== undefined ? chalk.gray(` [default: ${option.default}]`) : '';
        
        console.log(`  ${flags}${typeInfo}${required}`);
        console.log(`    ${option.description}${defaultValue}`);
        
        if (option.choices) {
          console.log(chalk.gray(`    Choices: ${option.choices.join(', ')}`));
        }
        console.log('');
      });
    }

    if (command.examples && command.examples.length > 0) {
      console.log(chalk.yellow('Examples:'));
      command.examples.forEach(example => {
        console.log(`  ${chalk.green(example.command)}`);
        console.log(`    ${example.description}`);
        console.log('');
      });
    }
  }

  private getCommandCategory(command: CLICommand): string {
    // Categorize commands based on naming conventions
    if (command.name.includes('init') || command.name.includes('create')) return 'Project';
    if (command.name.includes('compile') || command.name.includes('build')) return 'Build';
    if (command.name.includes('test')) return 'Testing';
    if (command.name.includes('deploy')) return 'Deployment';
    if (command.name.includes('verify')) return 'Verification';
    if (command.name.includes('debug')) return 'Debug';
    return 'General';
  }
}

// Helper Classes

class ConsoleLogger implements CLILogger {
  private level: string = 'info';
  private colors: boolean = true;

  setLevel(level: string): void {
    this.level = level;
  }

  info(message: string): void {
    if (this.shouldLog('info')) {
      const prefix = this.colors ? chalk.blue('ℹ') : '[INFO]';
      console.log(prefix, message);
    }
  }

  warn(message: string): void {
    if (this.shouldLog('warn')) {
      const prefix = this.colors ? chalk.yellow('⚠') : '[WARN]';
      console.warn(prefix, message);
    }
  }

  error(message: string | Error): void {
    if (this.shouldLog('error')) {
      const msg = message instanceof Error ? message.message : message;
      const prefix = this.colors ? chalk.red('✖') : '[ERROR]';
      console.error(prefix, msg);
    }
  }

  success(message: string): void {
    if (this.shouldLog('info')) {
      const prefix = this.colors ? chalk.green('✓') : '[SUCCESS]';
      console.log(prefix, message);
    }
  }

  debug(message: string): void {
    if (this.shouldLog('debug')) {
      const prefix = this.colors ? chalk.gray('🔍') : '[DEBUG]';
      console.log(prefix, message);
    }
  }

  log(level: string, message: string): void {
    switch (level) {
      case 'info': this.info(message); break;
      case 'warn': this.warn(message); break;
      case 'error': this.error(message); break;
      case 'debug': this.debug(message); break;
      default: console.log(message);
    }
  }

  private shouldLog(level: string): boolean {
    const levels = ['error', 'warn', 'info', 'debug', 'trace'];
    return levels.indexOf(level) <= levels.indexOf(this.level);
  }
}

class SpinnerManager implements CLISpinner {
  private spinner?: Ora;
  public isSpinning: boolean = false;

  start(text: string): void {
    this.spinner = ora(text).start();
    this.isSpinning = true;
  }

  succeed(text?: string): void {
    if (this.spinner) {
      this.spinner.succeed(text);
      this.isSpinning = false;
    }
  }

  fail(text?: string): void {
    if (this.spinner) {
      this.spinner.fail(text);
      this.isSpinning = false;
    }
  }

  warn(text?: string): void {
    if (this.spinner) {
      this.spinner.warn(text);
      this.isSpinning = false;
    }
  }

  info(text?: string): void {
    if (this.spinner) {
      this.spinner.info(text);
      this.isSpinning = false;
    }
  }

  stop(): void {
    if (this.spinner) {
      this.spinner.stop();
      this.isSpinning = false;
    }
  }
}

class ProgressManager implements CLIProgress {
  private total: number = 0;
  private current: number = 0;

  start(total: number, initialValue: number = 0): void {
    this.total = total;
    this.current = initialValue;
    this.render();
  }

  update(current: number, _payload?: unknown): void {
    this.current = current;
    this.render();
  }

  increment(step: number = 1, _payload?: unknown): void {
    this.current += step;
    this.render();
  }

  stop(): void {
    console.log(''); // New line after progress bar
  }

  private render(): void {
    const percentage = Math.round((this.current / this.total) * 100);
    const filledBlocks = Math.round((this.current / this.total) * 20);
    const emptyBlocks = 20 - filledBlocks;
    
    const bar = '█'.repeat(filledBlocks) + '░'.repeat(emptyBlocks);
    const progress = `[${bar}] ${percentage}% (${this.current}/${this.total})`;
    
    process.stdout.write(`\r${progress}`);
  }
}

class InteractiveManager implements CLIInteractive {
  async confirm(message: string, defaultValue: boolean = false): Promise<boolean> {
    const { confirmed } = await inquirer.prompt([{
      type: 'confirm',
      name: 'confirmed',
      message,
      default: defaultValue
    }]);
    return confirmed;
  }

  async input(message: string, defaultValue?: string): Promise<string> {
    const { value } = await inquirer.prompt([{
      type: 'input',
      name: 'value',
      message,
      default: defaultValue
    }]);
    return value;
  }

  async password(message: string): Promise<string> {
    const { password } = await inquirer.prompt([{
      type: 'password',
      name: 'password',
      message,
      mask: '*'
    }]);
    return password;
  }

  async select<T = string>(message: string, choices: T[] | any[]): Promise<T> {
    const { selected } = await inquirer.prompt([{
      type: 'list',
      name: 'selected',
      message,
      choices: Array.isArray(choices[0]) ? choices : choices.map(c => ({ name: String(c), value: c }))
    }]);
    return selected;
  }

  async multiSelect<T = string>(message: string, choices: T[] | any[]): Promise<T[]> {
    const { selected } = await inquirer.prompt([{
      type: 'checkbox',
      name: 'selected',
      message,
      choices: Array.isArray(choices[0]) ? choices : choices.map(c => ({ name: String(c), value: c }))
    }]);
    return selected;
  }

  async autocomplete(message: string, _source: (input: string) => Promise<string[]>): Promise<string> {
    // This would require inquirer-autocomplete-prompt plugin
    const { value } = await inquirer.prompt([{
      type: 'input',
      name: 'value',
      message
    }]);
    return value;
  }
}

export class CLIFormatterImpl implements CLIFormatter {
  table(data: CLITable): string {
    const { headers, rows, options = {} } = data;
    const { padding = 2, alignment = [], colors = true, borders = true } = options;
    
    // Calculate column widths
    const widths = headers.map((header, i) => {
      const contentWidths = rows.map(row => String(row[i] || '').length);
      return Math.max(header.length, ...contentWidths);
    });

    // Create separator
    const separator = borders 
      ? `+${widths.map(w => '-'.repeat(w + padding * 2)).join('+')}+`
      : '';

    // Format header
    const formattedHeaders = headers.map((header, i) => {
      const aligned = this.alignText(header, widths[i], alignment[i] || 'left');
      return borders ? ` ${aligned.padEnd(widths[i] + padding)} ` : aligned;
    });

    const headerRow = borders 
      ? `|${formattedHeaders.join('|')}|`
      : formattedHeaders.join(' '.repeat(padding));

    // Format rows
    const formattedRows = rows.map(row => {
      const formattedCells = row.map((cell, i) => {
        const cellText = String(cell || '');
        const aligned = this.alignText(cellText, widths[i], alignment[i] || 'left');
        return borders ? ` ${aligned.padEnd(widths[i] + padding)} ` : aligned;
      });

      return borders 
        ? `|${formattedCells.join('|')}|`
        : formattedCells.join(' '.repeat(padding));
    });

    // Assemble table
    const lines = [];
    if (borders) lines.push(separator);
    if (colors) {
      lines.push(chalk.bold(headerRow));
    } else {
      lines.push(headerRow);
    }
    if (borders) lines.push(separator);
    lines.push(...formattedRows);
    if (borders) lines.push(separator);

    return lines.join('\n');
  }

  json(data: any, pretty: boolean = true): string {
    return pretty ? JSON.stringify(data, null, 2) : JSON.stringify(data);
  }

  yaml(data: any): string {
    // This would use a YAML library like 'js-yaml'
    return JSON.stringify(data, null, 2); // Fallback to JSON
  }

  csv(data: any[]): string {
    if (data.length === 0) return '';
    
    const headers = Object.keys(data[0]);
    const csvRows = [
      headers.join(','),
      ...data.map(row => 
        headers.map(header => this.escapeCsvValue(row[header])).join(',')
      )
    ];

    return csvRows.join('\n');
  }

  tree(data: any, options: any = {}): string {
    const { unicode = true, colors = true, maxDepth = 10 } = options;
    const symbols = unicode 
      ? { branch: '├── ', lastBranch: '└── ', vertical: '│   ', space: '    ' }
      : { branch: '|-- ', lastBranch: '\\-- ', vertical: '|   ', space: '    ' };

    return this.renderTreeNode(data, '', true, symbols, colors, 0, maxDepth);
  }

  private alignText(text: string, width: number, alignment: 'left' | 'center' | 'right'): string {
    switch (alignment) {
      case 'center':
        {
          const padding = Math.max(0, width - text.length);
          const leftPad = Math.floor(padding / 2);
          const rightPad = padding - leftPad;
          return ' '.repeat(leftPad) + text + ' '.repeat(rightPad);
        }
      case 'right':
        return text.padStart(width);
      default:
        return text.padEnd(width);
    }
  }

  private escapeCsvValue(value: any): string {
    const str = String(value || '');
    if (str.includes(',') || str.includes('"') || str.includes('\n')) {
      return `"${str.replace(/"/g, '""')}"`;
    }
    return str;
  }

  private renderTreeNode(
    node: any, 
    prefix: string, 
    isLast: boolean, 
    symbols: any, 
    colors: boolean, 
    depth: number, 
    maxDepth: number
  ): string {
    if (depth >= maxDepth) {
      return '';
    }

    const lines: string[] = [];
    const connector = isLast ? symbols.lastBranch : symbols.branch;
    const nodeStr = colors ? chalk.cyan(String(node.name || node.key || node)) : String(node.name || node.key || node);
    
    lines.push(prefix + connector + nodeStr);

    if (node.children && Array.isArray(node.children)) {
      const newPrefix = prefix + (isLast ? symbols.space : symbols.vertical);
      node.children.forEach((child: any, index: number) => {
        const childIsLast = index === node.children.length - 1;
        lines.push(this.renderTreeNode(child, newPrefix, childIsLast, symbols, colors, depth + 1, maxDepth));
      });
    }

    return lines.join('\n');
  }
}
