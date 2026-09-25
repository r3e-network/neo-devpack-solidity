import { isNeoAddress, neoAddressToScriptHash, strip0x } from "@neo-devpack-solidity/types";

const OP_PUSHINT8 = 0x00;
const OP_PUSHT = 0x08;
const OP_PUSHF = 0x09;
const OP_PUSHNULL = 0x0b;
const OP_PUSHDATA1 = 0x0c;
const OP_PUSHDATA2 = 0x0d;
const OP_PUSHDATA4 = 0x0e;
const OP_PUSHM1 = 0x0f;
const OP_PUSH0 = 0x10;
const OP_SYSCALL = 0x41;
const OP_PACKMAP = 0xbe;
const OP_PACK = 0xc0;
const OP_NEWARRAY0 = 0xc2;

const SYSTEM_CONTRACT_CALL = "627d5b52";
const CALLFLAGS_ALL = 15;

type ContractParamLike = any;

function pushOpcode(script: number[], opcode: number, operandHex?: string): void {
  script.push(opcode);
  if (operandHex) {
    script.push(...Buffer.from(operandHex, "hex"));
  }
}

function emitHexBytes(script: number[], littleEndianHex: string): void {
  const hex = strip0x(littleEndianHex);
  const size = hex.length / 2;
  if (size < 0x100) {
    pushOpcode(script, OP_PUSHDATA1, size.toString(16).padStart(2, "0") + hex);
    return;
  }
  if (size < 0x10000) {
    const len = Buffer.allocUnsafe(2);
    len.writeUInt16LE(size, 0);
    pushOpcode(script, OP_PUSHDATA2, len.toString("hex") + hex);
    return;
  }
  const len = Buffer.allocUnsafe(4);
  len.writeUInt32LE(size, 0);
  pushOpcode(script, OP_PUSHDATA4, len.toString("hex") + hex);
}

function emitBytes(script: number[], bytes: Buffer): void {
  emitHexBytes(script, bytes.toString("hex"));
}

function emitString(script: number[], value: string): void {
  emitBytes(script, Buffer.from(value, "utf8"));
}

function minimalSignedByteWidth(value: bigint): number {
  const widths = [1, 2, 4, 8, 16, 32];
  for (const width of widths) {
    const bits = BigInt(width * 8);
    const min = -(1n << (bits - 1n));
    const max = (1n << (bits - 1n)) - 1n;
    if (value >= min && value <= max) {
      return width;
    }
  }
  throw new Error(`Integer too large to emit: ${value.toString()}`);
}

function emitNumber(script: number[], value: bigint): void {
  if (value === -1n) {
    pushOpcode(script, OP_PUSHM1);
    return;
  }
  if (value >= 0n && value <= 16n) {
    pushOpcode(script, OP_PUSH0 + Number(value));
    return;
  }

  const width = minimalSignedByteWidth(value);
  const unsigned = BigInt.asUintN(width * 8, value);
  const bytes = Buffer.alloc(width);
  for (let i = 0; i < width; i++) {
    bytes[i] = Number((unsigned >> BigInt(i * 8)) & 0xffn);
  }

  pushOpcode(script, OP_PUSHINT8 + [1, 2, 4, 8, 16, 32].indexOf(width), bytes.toString("hex"));
}

function emitBoolean(script: number[], value: boolean): void {
  pushOpcode(script, value ? OP_PUSHT : OP_PUSHF);
}

function isHexStringLike(value: unknown): value is { toLittleEndian: () => string } {
  return !!value && typeof value === "object" && typeof (value as any).toLittleEndian === "function";
}

function normalizeHash160Value(value: string): string {
  if (isNeoAddress(value)) {
    return strip0x(neoAddressToScriptHash(value));
  }
  return strip0x(value);
}

function emitParam(script: number[], param: ContractParamLike): void {
  const type = String(param?.type ?? "");
  const value = param?.value;

  switch (type) {
    case "Any":
      if (value == null) {
        pushOpcode(script, OP_PUSHNULL);
      } else if (typeof value === "string") {
        emitHexBytes(script, strip0x(value));
      } else if (isHexStringLike(value)) {
        emitHexBytes(script, value.toLittleEndian());
      } else {
        throw new Error(`Unsupported Any value: ${typeof value}`);
      }
      return;
    case "Boolean":
      emitBoolean(script, Boolean(value));
      return;
    case "Integer":
      emitNumber(script, BigInt(value));
      return;
    case "String":
      emitString(script, String(value));
      return;
    case "ByteArray":
      if (isHexStringLike(value)) {
        emitHexBytes(script, value.toLittleEndian());
      } else if (typeof value === "string") {
        emitHexBytes(script, strip0x(value));
      } else {
        throw new Error("Unsupported ByteArray value");
      }
      return;
    case "Hash160":
      if (isHexStringLike(value)) {
        emitHexBytes(script, value.toLittleEndian());
      } else {
        emitHexBytes(script, normalizeHash160Value(String(value)));
      }
      return;
    case "Hash256":
      if (isHexStringLike(value)) {
        emitHexBytes(script, value.toLittleEndian());
      } else {
        const hex = strip0x(String(value));
        emitHexBytes(script, Buffer.from(hex, "hex").reverse().toString("hex"));
      }
      return;
    case "Array": {
      const items = Array.isArray(value) ? value : [];
      for (let i = items.length - 1; i >= 0; i--) {
        emitPush(script, items[i]);
      }
      emitNumber(script, BigInt(items.length));
      pushOpcode(script, OP_PACK);
      return;
    }
    case "Map": {
      const entries = Array.isArray(value) ? value : [];
      for (const entry of entries) {
        emitPush(script, entry.value);
        emitPush(script, entry.key);
      }
      emitNumber(script, BigInt(entries.length));
      pushOpcode(script, OP_PACKMAP);
      return;
    }
    default:
      throw new Error(`Unsupported contract parameter type: ${type}`);
  }
}

function emitPush(script: number[], value: any): void {
  if (Array.isArray(value)) {
    for (let i = value.length - 1; i >= 0; i--) {
      emitPush(script, value[i]);
    }
    emitNumber(script, BigInt(value.length));
    pushOpcode(script, OP_PACK);
    return;
  }

  switch (typeof value) {
    case "boolean":
      emitBoolean(script, value);
      return;
    case "number":
      emitNumber(script, BigInt(Math.trunc(value)));
      return;
    case "bigint":
      emitNumber(script, value);
      return;
    case "string":
      emitString(script, value);
      return;
    case "object":
      if (value == null) {
        pushOpcode(script, OP_PUSHNULL);
        return;
      }
      if (isHexStringLike(value)) {
        emitHexBytes(script, value.toLittleEndian());
        return;
      }
      if (typeof value.type === "string") {
        emitParam(script, value);
        return;
      }
      throw new Error(`Unsupported object in script builder: ${value}`);
    default:
      throw new Error(`Unsupported push value: ${typeof value}`);
  }
}

export function createContractCallScript(input: {
  scriptHash: string;
  operation: string;
  args?: ContractParamLike[];
  callFlags?: number;
}): string {
  const script: number[] = [];
  const args = input.args ?? [];

  if (args.length === 0) {
    pushOpcode(script, OP_NEWARRAY0);
  } else {
    for (let i = args.length - 1; i >= 0; i--) {
      emitPush(script, args[i]);
    }
    emitNumber(script, BigInt(args.length));
    pushOpcode(script, OP_PACK);
  }

  emitNumber(script, BigInt(input.callFlags ?? CALLFLAGS_ALL));
  emitString(script, input.operation);
  emitHexBytes(script, Buffer.from(strip0x(input.scriptHash), "hex").reverse().toString("hex"));
  pushOpcode(script, OP_SYSCALL, SYSTEM_CONTRACT_CALL);

  return Buffer.from(script).toString("hex");
}
