import http from "http";
import { promises as fs, readFileSync } from "fs";
import os from "os";
import path from "path";

export interface NeoAnvilStatus {
  running: boolean;
  port?: number;
  chainId?: number;
  accounts?: number;
}

export interface NeoAnvilStartOptions {
  port?: number;
  chainId?: number;
  accounts?: number;
  balance?: string;
  gasLimit?: string;
  gasPrice?: string;
  blockTime?: number;
  fork?: string;
  forkBlockNumber?: number;
  quiet?: boolean;
}

type AnvilState = {
  blockHeight: number;
  time: number;
  snapshots: Map<string, { blockHeight: number; time: number }>;
};

const DEFAULT_PORT = 40332;
const STATE_FILE = path.join(os.tmpdir(), "neo-anvil.json");

function nowSeconds(): number {
  return Math.floor(Date.now() / 1000);
}

async function readStateFile(): Promise<NeoAnvilStatus | null> {
  try {
    const raw = await fs.readFile(STATE_FILE, "utf8");
    const parsed = JSON.parse(raw) as NeoAnvilStatus;
    if (!parsed?.port) return null;
    return parsed;
  } catch {
    return null;
  }
}

function readStateFileSync(): NeoAnvilStatus | null {
  try {
    const raw = readFileSync(STATE_FILE, "utf8");
    const parsed = JSON.parse(raw) as NeoAnvilStatus;
    if (!parsed?.port) return null;
    return parsed;
  } catch {
    return null;
  }
}

async function writeStateFile(status: NeoAnvilStatus): Promise<void> {
  await fs.writeFile(STATE_FILE, JSON.stringify(status, null, 2), "utf8");
}

async function removeStateFile(): Promise<void> {
  try {
    await fs.unlink(STATE_FILE);
  } catch {
    // ignore
  }
}

async function rpcControlCall(
  port: number,
  method: string,
  params: unknown[] = []
): Promise<unknown> {
  const body = JSON.stringify({
    jsonrpc: "2.0",
    method,
    params,
    id: 1,
  });

  return new Promise((resolve, reject) => {
    const req = http.request(
      {
        hostname: "127.0.0.1",
        port,
        path: "/",
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(body),
        },
      },
      (res) => {
        const chunks: Buffer[] = [];
        res.on("data", (c) => chunks.push(Buffer.isBuffer(c) ? c : Buffer.from(c)));
        res.on("end", () => {
          const text = Buffer.concat(chunks).toString("utf8");
          try {
            const parsed = JSON.parse(text) as { result?: unknown; error?: unknown };
            if (parsed?.error) {
              reject(new Error(`Neo-Anvil RPC error: ${JSON.stringify(parsed.error)}`));
              return;
            }
            resolve(parsed?.result);
          } catch (err) {
            reject(err);
          }
        });
      }
    );

    req.on("error", reject);
    req.write(body);
    req.end();
  });
}

/**
 * Neo-Anvil - in-memory mock chain with a minimal JSON-RPC façade.
 *
 * This is intentionally limited: it exists to support CLI demos and local
 * development ergonomics, not to execute real NEF bytecode.
 */
export class NeoAnvil {
  private server?: http.Server;
  private status: NeoAnvilStatus = { running: false };
  private state: AnvilState = {
    blockHeight: 0,
    time: nowSeconds(),
    snapshots: new Map(),
  };

  async start(options: NeoAnvilStartOptions = {}): Promise<void> {
    if (this.server) {
      throw new Error("Neo-Anvil is already running in this process.");
    }

    const port = options.port ?? DEFAULT_PORT;
    const chainId = options.chainId ?? 12345;
    const accounts = options.accounts ?? 10;

    this.state = {
      blockHeight: 0,
      time: nowSeconds(),
      snapshots: new Map(),
    };

    this.server = http.createServer(async (req, res) => {
      if (req.method !== "POST") {
        res.writeHead(405);
        res.end();
        return;
      }

      const chunks: Buffer[] = [];
      req.on("data", (c) => chunks.push(Buffer.isBuffer(c) ? c : Buffer.from(c)));
      req.on("end", async () => {
        try {
          const payload = JSON.parse(Buffer.concat(chunks).toString("utf8")) as {
            id?: number | string | null;
            method?: string;
            params?: unknown[];
          };

          const id = payload.id ?? 1;
          const method = payload.method ?? "";
          const params = payload.params ?? [];

          const respond = (result: unknown) => {
            res.writeHead(200, { "Content-Type": "application/json" });
            res.end(JSON.stringify({ jsonrpc: "2.0", id, result }));
          };

          const respondError = (message: string) => {
            res.writeHead(200, { "Content-Type": "application/json" });
            res.end(
              JSON.stringify({
                jsonrpc: "2.0",
                id,
                error: { code: -32000, message },
              })
            );
          };

          // Control methods (used by the CLI to talk to a running instance).
          if (method === "neo_anvil_stop") {
            respond(true);
            setImmediate(() => this.stop().catch(() => {}));
            return;
          }
          if (method === "neo_anvil_reset") {
            this.state.blockHeight = 0;
            this.state.time = nowSeconds();
            this.state.snapshots.clear();
            respond(true);
            return;
          }
          if (method === "neo_anvil_mine") {
            const count = Number(params[0] ?? 1);
            this.state.blockHeight += Number.isFinite(count) ? Math.max(0, count) : 1;
            respond(true);
            return;
          }
          if (method === "neo_anvil_setTime") {
            const timestamp = Number(params[0]);
            if (!Number.isFinite(timestamp)) {
              respondError("Invalid timestamp");
              return;
            }
            this.state.time = Math.floor(timestamp);
            respond(true);
            return;
          }
          if (method === "neo_anvil_increaseTime") {
            const seconds = Number(params[0]);
            if (!Number.isFinite(seconds)) {
              respondError("Invalid seconds");
              return;
            }
            this.state.time += Math.floor(seconds);
            respond(true);
            return;
          }
          if (method === "neo_anvil_snapshot") {
            const snapshotId = `${Date.now()}-${Math.random().toString(16).slice(2)}`;
            this.state.snapshots.set(snapshotId, {
              blockHeight: this.state.blockHeight,
              time: this.state.time,
            });
            respond(snapshotId);
            return;
          }
          if (method === "neo_anvil_restore") {
            const snapshotId = String(params[0] ?? "");
            const snap = this.state.snapshots.get(snapshotId);
            if (!snap) {
              respondError(`Unknown snapshot id: ${snapshotId}`);
              return;
            }
            this.state.blockHeight = snap.blockHeight;
            this.state.time = snap.time;
            respond(true);
            return;
          }

          // Minimal read-only Neo N3-like RPC surface (enough for demos).
          if (method === "getversion") {
            respond({
              useragent: "neo-anvil/0.1.0",
              protocol: { network: chainId, validatorscount: 0 },
            });
            return;
          }
          if (method === "getpeers") {
            respond({ connected: [] });
            return;
          }
          if (method === "getblockcount") {
            respond(this.state.blockHeight);
            return;
          }
          if (method === "getbestblockhash") {
            respond("0x" + "00".repeat(32));
            return;
          }
          if (method === "invokefunction") {
            respond({
              state: "HALT",
              gasConsumed: "0",
              stack: [],
            });
            return;
          }
          if (method === "getnep17balances") {
            respond({ balance: [] });
            return;
          }

          respondError(`Method not implemented: ${method}`);
        } catch (err) {
          res.writeHead(200, { "Content-Type": "application/json" });
          res.end(
            JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              error: { code: -32700, message: err instanceof Error ? err.message : String(err) },
            })
          );
        }
      });
    });

    await new Promise<void>((resolve, reject) => {
      this.server!.once("error", reject);
      this.server!.listen(port, "127.0.0.1", () => resolve());
    });

    this.status = { running: true, port, chainId, accounts };
    await writeStateFile(this.status);

    if (!options.quiet) {
      // The CLI prints its own UX; keep this minimal.
      // eslint-disable-next-line no-console
      console.log(`Neo-Anvil listening on http://127.0.0.1:${port} (chainId=${chainId})`);
    }
  }

  async stop(): Promise<void> {
    if (!this.server) {
      const running = await readStateFile();
      const port = running?.port ?? DEFAULT_PORT;
      await rpcControlCall(port, "neo_anvil_stop", []).catch(() => {});
      await removeStateFile();
      this.status = { running: false };
      return;
    }

    await new Promise<void>((resolve) => this.server!.close(() => resolve()));
    this.server = undefined;
    this.status = { running: false };
    await removeStateFile();
  }

  async reset(): Promise<void> {
    if (this.server) {
      this.state.blockHeight = 0;
      this.state.time = nowSeconds();
      this.state.snapshots.clear();
      return;
    }
    const running = await readStateFile();
    const port = running?.port ?? DEFAULT_PORT;
    await rpcControlCall(port, "neo_anvil_reset", []);
  }

  async mine(blocks = 1): Promise<void> {
    if (this.server) {
      this.state.blockHeight += Math.max(0, Math.floor(blocks));
      return;
    }
    const running = await readStateFile();
    const port = running?.port ?? DEFAULT_PORT;
    await rpcControlCall(port, "neo_anvil_mine", [blocks]);
  }

  async setTime(timestamp: number): Promise<void> {
    if (this.server) {
      this.state.time = Math.floor(timestamp);
      return;
    }
    const running = await readStateFile();
    const port = running?.port ?? DEFAULT_PORT;
    await rpcControlCall(port, "neo_anvil_setTime", [timestamp]);
  }

  async increaseTime(seconds: number): Promise<void> {
    if (this.server) {
      this.state.time += Math.floor(seconds);
      return;
    }
    const running = await readStateFile();
    const port = running?.port ?? DEFAULT_PORT;
    await rpcControlCall(port, "neo_anvil_increaseTime", [seconds]);
  }

  async snapshot(): Promise<string> {
    if (this.server) {
      const snapshotId = `${Date.now()}-${Math.random().toString(16).slice(2)}`;
      this.state.snapshots.set(snapshotId, {
        blockHeight: this.state.blockHeight,
        time: this.state.time,
      });
      return snapshotId;
    }
    const running = await readStateFile();
    const port = running?.port ?? DEFAULT_PORT;
    const result = await rpcControlCall(port, "neo_anvil_snapshot", []);
    return String(result);
  }

  async restore(snapshotId: string): Promise<void> {
    if (this.server) {
      const snap = this.state.snapshots.get(snapshotId);
      if (!snap) {
        throw new Error(`Unknown snapshot id: ${snapshotId}`);
      }
      this.state.blockHeight = snap.blockHeight;
      this.state.time = snap.time;
      return;
    }
    const running = await readStateFile();
    const port = running?.port ?? DEFAULT_PORT;
    await rpcControlCall(port, "neo_anvil_restore", [snapshotId]);
  }

  getStatus(): NeoAnvilStatus {
    if (this.status.running) return this.status;
    const fromDisk = readStateFileSync();
    return fromDisk ?? { running: false };
  }
}
