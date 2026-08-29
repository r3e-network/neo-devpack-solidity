import fs from "fs-extra";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { NeoDevpackSolidityCLI } from "../src/cli-framework";

const tempRoots: string[] = [];

afterEach(async () => {
  while (tempRoots.length > 0) {
    const dir = tempRoots.pop();
    if (dir) {
      await fs.remove(dir);
    }
  }
});

async function makeTempDir(prefix: string): Promise<string> {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempRoots.push(dir);
  return dir;
}

describe("@neo-devpack-solidity/cli-tools", () => {
  it("returns structured success and failure results for registered commands", async () => {
    const root = await makeTempDir("neo-cli-test-");
    const cli = new NeoDevpackSolidityCLI({
      defaults: {},
      profiles: {},
      plugins: [],
      aliases: {},
    });

    cli.register({
      name: "init",
      description: "Initialize a project",
      options: [
        { name: "template", description: "template", type: "string", required: true },
        { name: "name", description: "name", type: "string", required: true },
      ],
      action: async (args) => {
        await fs.ensureDir(path.join(root, String(args.name)));
      },
    });

    const success = await cli.execute(["node", "cli", "init", "erc20", "CLITestToken"]);
    expect(success.success).toBe(true);
    expect(await fs.pathExists(path.join(root, "CLITestToken"))).toBe(true);

    const failingCli = new NeoDevpackSolidityCLI({
      defaults: {},
      profiles: {},
      plugins: [],
      aliases: {},
    });

    failingCli.register({
      name: "fail",
      description: "Fails intentionally",
      options: [],
      action: async () => {
        throw new Error("Intentional failure");
      },
    });

    const failure = await failingCli.execute(["node", "cli", "fail"]);
    expect(failure.success).toBe(false);
    expect(failure.error?.message).toContain("Intentional failure");
  });
});
