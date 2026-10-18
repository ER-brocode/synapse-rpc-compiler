use byteorder::{BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use crate::error::{Result, SynapseError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    Fixed32 = 5,
}

impl WireType {
    pub fn from_u8(val: u8) -> Result<Self> {
        match val {
            0 => Ok(WireType::Varint),
            1 => Ok(WireType::Fixed64),
            2 => Ok(WireType::LengthDelimited),
            5 => Ok(WireType::Fixed32),
            other => Err(SynapseError::CodecError {
                message: format!("Unknown wire type: {}", other),
            }),
        }
    }
}

pub fn encode_zigzag32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

pub fn decode_zigzag32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 1) as i32))
}

pub fn encode_zigzag64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

pub fn decode_zigzag64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

#[derive(Default, Debug, Clone)]
pub struct SynapseEncoder {
    buffer: Vec<u8>,
}

impl SynapseEncoder {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn write_tag(&mut self, field_id: u32, wire_type: WireType) {
        let tag = (field_id << 3) | (wire_type as u32);
        self.write_varint_u32(tag);
    }

    pub fn write_varint_u32(&mut self, mut val: u32) {
        while val >= 0x80 {
            self.buffer.push(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
        self.buffer.push(val as u8);
    }

    pub fn write_varint_u64(&mut self, mut val: u64) {
        while val >= 0x80 {
            self.buffer.push(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
        self.buffer.push(val as u8);
    }

    pub fn write_bool(&mut self, field_id: u32, val: bool) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u32(if val { 1 } else { 0 });
    }

    pub fn write_u8(&mut self, field_id: u32, val: u8) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u32(val as u32);
    }

    pub fn write_u16(&mut self, field_id: u32, val: u16) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u32(val as u32);
    }

    pub fn write_u32(&mut self, field_id: u32, val: u32) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u32(val);
    }

    pub fn write_u64(&mut self, field_id: u32, val: u64) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u64(val);
    }

    pub fn write_i8(&mut self, field_id: u32, val: i8) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u32(encode_zigzag32(val as i32));
    }

    pub fn write_i16(&mut self, field_id: u32, val: i16) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u32(encode_zigzag32(val as i32));
    }

    pub fn write_i32(&mut self, field_id: u32, val: i32) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u32(encode_zigzag32(val));
    }

    pub fn write_i64(&mut self, field_id: u32, val: i64) {
        self.write_tag(field_id, WireType::Varint);
        self.write_varint_u64(encode_zigzag64(val));
    }

    pub fn write_f32(&mut self, field_id: u32, val: f32) {
        self.write_tag(field_id, WireType::Fixed32);
        let mut bytes = [0u8; 4];
        BigEndian::write_f32(&mut bytes, val);
        self.buffer.extend_from_slice(&bytes);
    }

    pub fn write_f64(&mut self, field_id: u32, val: f64) {
        self.write_tag(field_id, WireType::Fixed64);
        let mut bytes = [0u8; 8];
        BigEndian::write_f64(&mut bytes, val);
        self.buffer.extend_from_slice(&bytes);
    }

    pub fn write_string(&mut self, field_id: u32, val: &str) {
        self.write_tag(field_id, WireType::LengthDelimited);
        let bytes = val.as_bytes();
        self.write_varint_u32(bytes.len() as u32);
        self.buffer.extend_from_slice(bytes);
    }

    pub fn write_bytes(&mut self, field_id: u32, val: &[u8]) {
        self.write_tag(field_id, WireType::LengthDelimited);
        self.write_varint_u32(val.len() as u32);
        self.buffer.extend_from_slice(val);
    }

    pub fn write_nested_message(&mut self, field_id: u32, nested: &SynapseEncoder) {
        self.write_tag(field_id, WireType::LengthDelimited);
        self.write_varint_u32(nested.len() as u32);
        self.buffer.extend_from_slice(nested.as_slice());
    }
}

pub struct SynapseDecoder<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> SynapseDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn is_eof(&self) -> bool {
        self.cursor >= self.data.len()
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    pub fn read_tag(&mut self) -> Result<Option<(u32, WireType)>> {
        if self.is_eof() {
            return Ok(None);
        }
        let tag = self.read_varint_u32()?;
        let field_id = tag >> 3;
        let wire_type = WireType::from_u8((tag & 0x07) as u8)?;
        Ok(Some((field_id, wire_type)))
    }

    pub fn read_varint_u32(&mut self) -> Result<u32> {
        let val = self.read_varint_u64()?;
        if val > u32::MAX as u64 {
            return Err(SynapseError::CodecError {
                message: "Varint overflows u32".to_string(),
            });
        }
        Ok(val as u32)
    }

    pub fn read_varint_u64(&mut self) -> Result<u64> {
        let mut result: u64 = 0;
        let mut shift = 0;

        while shift < 64 {
            if self.is_eof() {
                return Err(SynapseError::CodecError {
                    message: "Unexpected end of input reading varint".to_string(),
                });
            }
            let byte = self.data[self.cursor];
            self.cursor += 1;

            result |= ((byte & 0x7F) as u64) << shift;
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
            shift += 7;
        }

        Err(SynapseError::CodecError {
            message: "Varint exceeded 64 bits".to_string(),
        })
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        let v = self.read_varint_u32()?;
        Ok(v != 0)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let v = self.read_varint_u32()?;
        Ok(v as u8)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let v = self.read_varint_u32()?;
        Ok(v as u16)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        self.read_varint_u32()
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        self.read_varint_u64()
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        let v = self.read_varint_u32()?;
        Ok(decode_zigzag32(v) as i8)
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let v = self.read_varint_u32()?;
        Ok(decode_zigzag32(v) as i16)
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let v = self.read_varint_u32()?;
        Ok(decode_zigzag32(v))
    }

    pub fn read_i64(&mut self) -> Result<i64> {
        let v = self.read_varint_u64()?;
        Ok(decode_zigzag64(v))
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        if self.remaining() < 4 {
            return Err(SynapseError::CodecError {
                message: "Unexpected end of input reading f32".to_string(),
            });
        }
        let val = BigEndian::read_f32(&self.data[self.cursor..self.cursor + 4]);
        self.cursor += 4;
        Ok(val)
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        if self.remaining() < 8 {
            return Err(SynapseError::CodecError {
                message: "Unexpected end of input reading f64".to_string(),
            });
        }
        let val = BigEndian::read_f64(&self.data[self.cursor..self.cursor + 8]);
        self.cursor += 8;
        Ok(val)
    }

    pub fn read_string(&mut self) -> Result<String> {
        let len = self.read_varint_u32()? as usize;
        if self.remaining() < len {
            return Err(SynapseError::CodecError {
                message: format!("Length prefix {} exceeds buffer length {}", len, self.remaining()),
            });
        }
        let bytes = &self.data[self.cursor..self.cursor + len];
        self.cursor += len;
        String::from_utf8(bytes.to_vec()).map_err(|e| SynapseError::CodecError {
            message: format!("Invalid UTF-8 string: {}", e),
        })
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>> {
        let len = self.read_varint_u32()? as usize;
        if self.remaining() < len {
            return Err(SynapseError::CodecError {
                message: format!("Length prefix {} exceeds buffer length {}", len, self.remaining()),
            });
        }
        let bytes = &self.data[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(bytes.to_vec())
    }

    pub fn read_nested_decoder(&mut self) -> Result<SynapseDecoder<'a>> {
        let len = self.read_varint_u32()? as usize;
        if self.remaining() < len {
            return Err(SynapseError::CodecError {
                message: format!("Nested length prefix {} exceeds buffer length {}", len, self.remaining()),
            });
        }
        let slice = &self.data[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(SynapseDecoder::new(slice))
    }

    pub fn skip_field(&mut self, wire_type: WireType) -> Result<()> {
        match wire_type {
            WireType::Varint => {
                self.read_varint_u64()?;
            }
            WireType::Fixed64 => {
                if self.remaining() < 8 {
                    return Err(SynapseError::CodecError { message: "Cannot skip fixed64".to_string() });
                }
                self.cursor += 8;
            }
            WireType::Fixed32 => {
                if self.remaining() < 4 {
                    return Err(SynapseError::CodecError { message: "Cannot skip fixed32".to_string() });
                }
                self.cursor += 4;
            }
            WireType::LengthDelimited => {
                let len = self.read_varint_u32()? as usize;
                if self.remaining() < len {
                    return Err(SynapseError::CodecError { message: "Cannot skip length-delimited field".to_string() });
                }
                self.cursor += len;
            }
        }
        Ok(())
    }
}

pub trait SynapseCodec: Sized {
    fn encode(&self, encoder: &mut SynapseEncoder) -> Result<()>;
    fn decode(decoder: &mut SynapseDecoder) -> Result<Self>;

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut encoder = SynapseEncoder::new();
        self.encode(&mut encoder)?;
        Ok(encoder.into_bytes())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut decoder = SynapseDecoder::new(bytes);
        Self::decode(&mut decoder)
    }
}
