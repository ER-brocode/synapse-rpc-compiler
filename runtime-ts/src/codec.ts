export enum WireType {
  Varint = 0,
  Fixed64 = 1,
  LengthDelimited = 2,
  Fixed32 = 5,
}

export function encodeZigzag32(n: number): number {
  return ((n << 1) ^ (n >> 31)) >>> 0;
}

export function decodeZigzag32(n: number): number {
  return (n >>> 1) ^ -(n & 1);
}

export function encodeZigzag64(n: bigint): bigint {
  return (n << 1n) ^ (n >> 63n);
}

export function decodeZigzag64(n: bigint): bigint {
  return (n >> 1n) ^ -(n & 1n);
}

export class SynapseEncoder {
  private buffer: number[] = [];

  intoBytes(): Uint8Array {
    return new Uint8Array(this.buffer);
  }

  writeVarintU32(val: number): void {
    let v = val >>> 0;
    while (v >= 0x80) {
      this.buffer.push((v & 0x7f) | 0x80);
      v >>>= 7;
    }
    this.buffer.push(v);
  }

  writeVarintU64(val: bigint): void {
    let v = val;
    while (v >= 0x80n) {
      this.buffer.push(Number(v & 0x7fn) | 0x80);
      v >>= 7n;
    }
    this.buffer.push(Number(v));
  }

  writeTag(fieldId: number, wireType: WireType): void {
    const tag = ((fieldId << 3) | wireType) >>> 0;
    this.writeVarintU32(tag);
  }

  writeBool(fieldId: number, val: boolean): void {
    this.writeTag(fieldId, WireType.Varint);
    this.writeVarintU32(val ? 1 : 0);
  }

  writeU32(fieldId: number, val: number): void {
    this.writeTag(fieldId, WireType.Varint);
    this.writeVarintU32(val);
  }

  writeI32(fieldId: number, val: number): void {
    this.writeTag(fieldId, WireType.Varint);
    this.writeVarintU32(encodeZigzag32(val));
  }

  writeString(fieldId: number, val: string): void {
    this.writeTag(fieldId, WireType.LengthDelimited);
    const bytes = new TextEncoder().encode(val);
    this.writeVarintU32(bytes.length);
    for (let i = 0; i < bytes.length; i++) {
      this.buffer.push(bytes[i]);
    }
  }

  writeBytes(fieldId: number, bytes: Uint8Array): void {
    this.writeTag(fieldId, WireType.LengthDelimited);
    this.writeVarintU32(bytes.length);
    for (let i = 0; i < bytes.length; i++) {
      this.buffer.push(bytes[i]);
    }
  }
}

export class SynapseDecoder {
  private cursor = 0;

  constructor(private data: Uint8Array) {}

  isEof(): boolean {
    return this.cursor >= this.data.length;
  }

  readVarintU32(): number {
    let result = 0;
    let shift = 0;
    while (shift < 32) {
      if (this.isEof()) throw new Error('Unexpected EOF reading varint');
      const byte = this.data[this.cursor++];
      result |= (byte & 0x7f) << shift;
      if ((byte & 0x80) === 0) return result >>> 0;
      shift += 7;
    }
    throw new Error('Varint overflow u32');
  }

  readTag(): { fieldId: number; wireType: WireType } | null {
    if (this.isEof()) return null;
    const tag = this.readVarintU32();
    return {
      fieldId: tag >>> 3,
      wireType: (tag & 0x07) as WireType,
    };
  }

  readBool(): boolean {
    return this.readVarintU32() !== 0;
  }

  readU32(): number {
    return this.readVarintU32();
  }

  readI32(): number {
    return decodeZigzag32(this.readVarintU32());
  }

  readString(): string {
    const len = this.readVarintU32();
    const slice = this.data.slice(this.cursor, this.cursor + len);
    this.cursor += len;
    return new TextDecoder().decode(slice);
  }

  readBytes(): Uint8Array {
    const len = this.readVarintU32();
    const slice = this.data.slice(this.cursor, this.cursor + len);
    this.cursor += len;
    return slice;
  }
}
