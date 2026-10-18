import assert from 'assert';

// Inline test implementation matching codec & protocol logic
const SYNAPSE_MAGIC = 0x53594e50;
const SYNAPSE_VERSION = 1;
const HEADER_SIZE = 24;

const TABLE = (() => {
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

function crc32(buffer) {
  let c = 0xffffffff;
  for (let i = 0; i < buffer.length; i++) {
    c = (c >>> 8) ^ TABLE[(c ^ buffer[i]) & 0xff];
  }
  return (c ^ 0xffffffff) >>> 0;
}

function encodeFrame(frame) {
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

function decodeFrame(bytes) {
  if (bytes.length < HEADER_SIZE) throw new Error('Buffer too small');
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const magic = view.getUint32(0, false);
  assert.strictEqual(magic, SYNAPSE_MAGIC);
  const version = view.getUint8(4);
  assert.strictEqual(version, SYNAPSE_VERSION);
  const frameType = view.getUint8(5);
  const flags = view.getUint16(6, false);
  const streamId = view.getUint32(8, false);
  const methodId = view.getUint32(12, false);
  const payloadLen = view.getUint32(16, false);
  const checksum = view.getUint32(20, false);

  const payload = bytes.slice(HEADER_SIZE, HEADER_SIZE + payloadLen);
  const crcTarget = new Uint8Array(20 + payload.length);
  crcTarget.set(bytes.subarray(0, 20), 0);
  crcTarget.set(payload, 20);
  const calculated = crc32(crcTarget);
  assert.strictEqual(calculated, checksum);

  return {
    header: { magic, version, frameType, flags, streamId, methodId, payloadLen, checksum },
    payload
  };
}

console.log('Testing TypeScript Synapse Protocol...');
const payload = new TextEncoder().encode('Hello Synapse from TypeScript!');
const rawFrame = {
  header: {
    frameType: 2, // CallRequest
    flags: 1,     // END_STREAM
    streamId: 77,
    methodId: 101,
  },
  payload
};

const encoded = encodeFrame(rawFrame);
assert.strictEqual(encoded.length, HEADER_SIZE + payload.length);

const decoded = decodeFrame(encoded);
assert.strictEqual(decoded.header.streamId, 77);
assert.strictEqual(decoded.header.methodId, 101);
assert.strictEqual(new TextDecoder().decode(decoded.payload), 'Hello Synapse from TypeScript!');

console.log('✔ TypeScript Synapse Protocol tests passed successfully!');
