export interface FormatterOptions {
  prefix?: string;
  colorize?: boolean;
}

export class NeoFoundryFormatter {
  constructor(private readonly options: FormatterOptions = {}) {}

  format(message: string): string {
    const prefixed = this.options.prefix
      ? `${this.options.prefix} ${message}`
      : message;
    return this.options.colorize ? `\x1b[36m${prefixed}\x1b[0m` : prefixed;
  }
}
