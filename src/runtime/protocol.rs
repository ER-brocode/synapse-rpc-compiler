use byteorder::{BigEndian, ByteOrder};
use crc32fast::Hasher;
use crate::error::{Result, SynapseError};

pub const SYNAPSE_MAGIC: u32 = 0x53594E50; // "SYNP"
pub const SYNAPSE_VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 24;

pub const FLAG_END_STREAM: u16 = 0x0001;
pub const FLAG_COMPRESSED: u16 = 0x0002;
pub const FLAG_ACK: u16 = 0x0004;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
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

impl FrameType {
    pub fn from_u8(v: u8) -> Result<Self> {
        match v {
            1 => Ok(FrameType::Handshake),
            2 => Ok(FrameType::CallRequest),
            3 => Ok(FrameType::CallResponse),
            4 => Ok(FrameType::StreamData),
            5 => Ok(FrameType::StreamClose),
            6 => Ok(FrameType::StreamCancel),
            7 => Ok(FrameType::Ping),
            8 => Ok(FrameType::Pong),
            9 => Ok(FrameType::Error),
            10 => Ok(FrameType::Credit),
            other => Err(SynapseError::ProtocolError {
                message: format!("Unknown frame type: {}", other),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub magic: u32,
    pub version: u8,
    pub frame_type: FrameType,
    pub flags: u16,
    pub stream_id: u32,
    pub method_id: u32,
    pub payload_len: u32,
    pub checksum: u32,
}

impl FrameHeader {
    pub fn new(frame_type: FrameType, stream_id: u32, method_id: u32, payload_len: u32) -> Self {
        Self {
            magic: SYNAPSE_MAGIC,
            version: SYNAPSE_VERSION,
            frame_type,
            flags: 0,
            stream_id,
            method_id,
            payload_len,
            checksum: 0,
        }
    }

    pub fn with_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    pub fn is_end_stream(&self) -> bool {
        (self.flags & FLAG_END_STREAM) != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(frame_type: FrameType, stream_id: u32, method_id: u32, payload: Vec<u8>) -> Self {
        let payload_len = payload.len() as u32;
        let mut header = FrameHeader::new(frame_type, stream_id, method_id, payload_len);
        let checksum = compute_crc32(&header, &payload);
        header.checksum = checksum;

        Self { header, payload }
    }

    pub fn with_flags(mut self, flags: u16) -> Self {
        self.header.flags = flags;
        self.header.checksum = compute_crc32(&self.header, &self.payload);
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = vec![0u8; HEADER_SIZE + self.payload.len()];
        BigEndian::write_u32(&mut buf[0..4], self.header.magic);
        buf[4] = self.header.version;
        buf[5] = self.header.frame_type as u8;
        BigEndian::write_u16(&mut buf[6..8], self.header.flags);
        BigEndian::write_u32(&mut buf[8..12], self.header.stream_id);
        BigEndian::write_u32(&mut buf[12..16], self.header.method_id);
        BigEndian::write_u32(&mut buf[16..20], self.header.payload_len);
        BigEndian::write_u32(&mut buf[20..24], self.header.checksum);
        buf[24..].copy_from_slice(&self.payload);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Frame> {
        if bytes.len() < HEADER_SIZE {
            return Err(SynapseError::ProtocolError {
                message: format!("Frame buffer too small: {} bytes (expected at least {})", bytes.len(), HEADER_SIZE),
            });
        }

        let magic = BigEndian::read_u32(&bytes[0..4]);
        if magic != SYNAPSE_MAGIC {
            return Err(SynapseError::ProtocolError {
                message: format!("Invalid magic bytes: 0x{:08X} (expected 0x{:08X})", magic, SYNAPSE_MAGIC),
            });
        }

        let version = bytes[4];
        if version != SYNAPSE_VERSION {
            return Err(SynapseError::ProtocolError {
                message: format!("Unsupported protocol version: {} (expected {})", version, SYNAPSE_VERSION),
            });
        }

        let frame_type = FrameType::from_u8(bytes[5])?;
        let flags = BigEndian::read_u16(&bytes[6..8]);
        let stream_id = BigEndian::read_u32(&bytes[8..12]);
        let method_id = BigEndian::read_u32(&bytes[12..16]);
        let payload_len = BigEndian::read_u32(&bytes[16..20]) as usize;
        let expected_checksum = BigEndian::read_u32(&bytes[20..24]);

        if bytes.len() < HEADER_SIZE + payload_len {
            return Err(SynapseError::ProtocolError {
                message: format!("Incomplete frame payload: got {} bytes, expected {}", bytes.len() - HEADER_SIZE, payload_len),
            });
        }

        let payload = bytes[HEADER_SIZE..HEADER_SIZE + payload_len].to_vec();

        let header = FrameHeader {
            magic,
            version,
            frame_type,
            flags,
            stream_id,
            method_id,
            payload_len: payload_len as u32,
            checksum: expected_checksum,
        };

        let calculated_checksum = compute_crc32(&header, &payload);
        if expected_checksum != calculated_checksum {
            return Err(SynapseError::ProtocolError {
                message: format!("CRC32 checksum mismatch: got 0x{:08X}, expected 0x{:08X}", calculated_checksum, expected_checksum),
            });
        }

        Ok(Frame { header, payload })
    }
}

fn compute_crc32(header: &FrameHeader, payload: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    let mut header_bytes = [0u8; 20];
    BigEndian::write_u32(&mut header_bytes[0..4], header.magic);
    header_bytes[4] = header.version;
    header_bytes[5] = header.frame_type as u8;
    BigEndian::write_u16(&mut header_bytes[6..8], header.flags);
    BigEndian::write_u32(&mut header_bytes[8..12], header.stream_id);
    BigEndian::write_u32(&mut header_bytes[12..16], header.method_id);
    BigEndian::write_u32(&mut header_bytes[16..20], header.payload_len);

    hasher.update(&header_bytes);
    hasher.update(payload);
    hasher.finalize()
}
