import { createHash } from "crypto";
import { promises as fs } from "fs";
import path from "path";

interface CacheEntry {
  hash: string;
  artifactPaths: string[];
}

interface CacheManifest {
  version: number;
  entries: Record<string, CacheEntry>;
}

const CACHE_VERSION = 1;

/**
 * Simple incremental build cache keyed by source file content hash.
 *
 * When `incremental` is enabled, `NeoForge.build` skips invoking the compiler
 * if every source file hash matches the previous build and all artifacts still
 * exist on disk.
 */
export class BuildCache {
  private readonly cacheDir: string;
  private readonly manifestPath: string;

  constructor(cacheDir: string) {
    this.cacheDir = cacheDir;
    this.manifestPath = path.join(cacheDir, "build-cache.json");
  }

  /**
   * Check whether all source files are up to date and their artifacts exist.
   */
  async isUpToDate(sourceFiles: string[], artifactPaths: string[]): Promise<boolean> {
    let manifest: CacheManifest;
    try {
      const content = await fs.readFile(this.manifestPath, "utf-8");
      manifest = JSON.parse(content) as CacheManifest;
      if (manifest.version !== CACHE_VERSION) {
        return false;
      }
    } catch {
      return false;
    }

    for (const artifactPath of artifactPaths) {
      try {
        await fs.access(artifactPath);
      } catch {
        return false;
      }
    }

    for (const sourceFile of sourceFiles) {
      const currentHash = await this.hashFile(sourceFile);
      const entry = manifest.entries[sourceFile];
      if (!entry || entry.hash !== currentHash) {
        return false;
      }
    }

    return true;
  }

  /**
   * Record the current source file hashes and produced artifact paths.
   */
  async update(sourceFiles: string[], artifactPaths: string[]): Promise<void> {
    await fs.mkdir(this.cacheDir, { recursive: true });

    const entries: Record<string, CacheEntry> = {};
    for (const sourceFile of sourceFiles) {
      entries[sourceFile] = {
        hash: await this.hashFile(sourceFile),
        artifactPaths,
      };
    }

    const manifest: CacheManifest = {
      version: CACHE_VERSION,
      entries,
    };

    await fs.writeFile(this.manifestPath, JSON.stringify(manifest, null, 2));
  }

  /**
   * Remove the cache manifest.
   */
  async clear(): Promise<void> {
    try {
      await fs.rm(this.manifestPath, { force: true });
    } catch {
      // ignore
    }
  }

  private async hashFile(filePath: string): Promise<string> {
    const content = await fs.readFile(filePath);
    return createHash("sha256").update(content).digest("hex");
  }
}
