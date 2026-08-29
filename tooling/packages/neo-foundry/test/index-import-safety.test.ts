import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const distEntrypoint = path.join(__dirname, "..", "dist", "index.js");

afterEach(() => {
  vi.restoreAllMocks();
});

describe("@neo-devpack-solidity/neo-foundry", () => {
  it("does not execute the CLI parser when importing the library entrypoint", async () => {
    const exitSpy = vi
      .spyOn(process, "exit")
      .mockImplementation(((code?: number) => {
        throw new Error(`process.exit unexpectedly called with ${code}`);
      }) as never);

    const module = await import("../src/index");

    expect(module.NeoForge).toBeDefined();
    expect(module.NeoCast).toBeDefined();
    expect(exitSpy).not.toHaveBeenCalled();
  });

  it("keeps the built dist entrypoint importable", async () => {
    if (!fs.existsSync(distEntrypoint)) {
      return;
    }

    const mod = await import("../dist/index.js");
    expect(mod.NeoForge).toBeDefined();
    expect(mod.NeoCast).toBeDefined();
  });
});
