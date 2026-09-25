import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const distEntrypoint = path.join(__dirname, "..", "dist", "index.js");

describe("@neo-devpack-solidity/cli-tools dist entrypoint", () => {
  it("is importable after build", async () => {
    if (!fs.existsSync(distEntrypoint)) {
      return;
    }

    const mod = await import("../dist/index.js");
    expect(mod).toBeDefined();
  });
});
