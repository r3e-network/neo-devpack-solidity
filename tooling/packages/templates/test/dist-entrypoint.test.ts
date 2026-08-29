import fs from "node:fs";
import fsExtra from "fs-extra";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const distEntrypoint = path.join(__dirname, "..", "dist", "index.js");

describe("@neo-devpack-solidity/templates dist entrypoint", () => {
  it("is importable after build", async () => {
    if (!fs.existsSync(distEntrypoint)) {
      return;
    }

    const mod = await import("../dist/index.js");
    expect(mod).toBeDefined();
  });

  it("can scaffold a project when loaded from dist", async () => {
    if (!fs.existsSync(distEntrypoint)) {
      return;
    }

    const { ProjectScaffolder } = await import("../dist/index.js");
    const tempDir = await fsExtra.mkdtemp(path.join(os.tmpdir(), "neo-template-dist-"));
    const projectPath = path.join(tempDir, "basic-project");
    const scaffolder = new ProjectScaffolder();

    const result = await scaffolder.scaffold({
      template: "basic",
      name: "BasicProject",
      directory: projectPath,
      context: {
        contractName: "BasicProject",
        author: "Neo DevPack for Solidity Team",
        description: "Basic Neo DevPack for Solidity project",
        license: "MIT",
        solcVersion: "0.8.19",
      },
      dryRun: false,
      interactive: false,
      gitInit: false,
      install: false,
    });

    expect(result.success).toBe(true);
    expect(fs.existsSync(path.join(projectPath, "package.json"))).toBe(true);
  });
});
