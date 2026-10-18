export const SYNAPSE_MAGIC = 0x53594e50; // "SYNP"
export const SYNAPSE_VERSION = 1;
export const HEADER_SIZE = 24;

export const FLAG_END_STREAM = 0x0001;
export const FLAG_COMPRESSED = 0x0002;
export const FLAG_ACK = 0x0004;

export enum FrameType {
  Handshake = 1,
  CallRequest = 2,
  CallResponse = 3,
  StreamData = 4,
  StreamClose = 5,
  StreamCancel = 6,
  Ping = 7,
  Pong = 8,
  Error = 9,
  Credit = 10,
}

export interface FrameHeader {
  magic: number;
  version: number;
  frameType: FrameType;
  flags: number;
  streamId: number;
  methodId: number;
  payloadLen: number;
  checksum: number;
}

export interface Frame {
  header: FrameHeader;
  payload: Uint8Array;
}

function crc32(buffer: Uint8Array): number {
  let c = 0xffffffff;
  for (let i = 0; i < buffer.length; i++) {
    c = (c >>> 8) ^ TABLE[(c ^ buffer[i]) & 0xff];
  }
  return (c ^ 0xffffffff) >>> 0;
}

const TABLE: number[] = (() => {
  const table = new Array(256);
  for (let i = 0; i < 256; i++) {
    let current = i;
    for (let bit = 0; bit < 8; bit++) {
      if ((current & 1) !== 0) {
        current = ((current >>> 1) ^ 0xedb88320) >>> 0;
      } else {
        current = (current >>> 1) >>> 0;
      }
    }
    table[i] = current;
  }
  return table;
})();

export function encodeFrame(frame: Frame): Uint8Array {
  const buf = new Uint8Array(HEADER_SIZE + frame.payload.length);
  const view = new DataView(buf.buffer);

  view.setUint32(0, SYNAPSE_MAGIC, false);
  view.setUint8(4, SYNAPSE_VERSION);
  view.setUint8(5, frame.header.frameType);
  view.setUint16(6, frame.header.flags, false);
  view.setUint32(8, frame.header.streamId, false);
  view.setUint32(12, frame.header.methodId, false);
  view.setUint32(16, frame.payload.length, false);

  const crcTarget = new Uint8Array(20 + frame.payload.length);
  crcTarget.set(buf.subarray(0, 20), 0);
  crcTarget.set(frame.payload, 20);
  const checksum = crc32(crcTarget);

  view.setUint32(20, checksum, false);
  buf.set(frame.payload, HEADER_SIZE);

  return buf;
}

export function decodeFrame(bytes: Uint8Array): Frame {
  if (bytes.length < HEADER_SIZE) {
    throw new Error(`Buffer too short: got ${bytes.length}, expected >= ${HEADER_SIZE}`);
  }

  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const magic = view.getUint32(0, false);
  if (magic !== SYNAPSE_MAGIC) {
    throw new Error(`Invalid magic: 0x${magic.toString(16)} (expected 0x${SYNAPSE_MAGIC.toString(16)})`);
  }

  const version = view.getUint8(4);
  const frameType = view.getUint8(5) as FrameType;
  const flags = view.getUint16(6, false);
  const streamId = view.getUint32(8, false);
  const methodId = view.getUint32(12, false);
  const payloadLen = view.getUint32(16, false);
  const checksum = view.getUint32(20, false);

  if (bytes.length < HEADER_SIZE + payloadLen) {
    throw new Error(`Incomplete payload: buffer has ${bytes.length - HEADER_SIZE}, expected ${payloadLen}`);
  }

  const payload = bytes.slice(HEADER_SIZE, HEADER_SIZE + payloadLen);

  const crcTarget = new Uint8Array(20 + payload.length);
  crcTarget.set(bytes.subarray(0, 20), 0);
  crcTarget.set(payload, 20);
  const calculated = crc32(crcTarget);

  if (calculated !== checksum) {
    throw new Error(`CRC32 mismatch: got 0x${calculated.toString(16)}, expected 0x${checksum.toString(16)}`);
  }

  return {
    header: {
      magic,
      version,
      frameType,
      flags,
      streamId,
      methodId,
      payloadLen,
      checksum,
    },
    payload,
  };
}
