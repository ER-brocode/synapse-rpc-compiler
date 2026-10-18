use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use crate::error::{Result, SynapseError};
use crate::runtime::protocol::{Frame, FrameType, FLAG_END_STREAM};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Idle,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
}

#[derive(Debug, Clone)]
pub struct StreamStatus {
    pub stream_id: u32,
    pub state: StreamState,
    pub method_id: u32,
    pub inbound_credit: u32,
    pub outbound_credit: u32,
}

pub struct Multiplexer {
    is_client: bool,
    next_stream_id: AtomicU32,
    streams: HashMap<u32, StreamStatus>,
    initial_credit: u32,
}

impl Multiplexer {
    pub const DEFAULT_INITIAL_CREDIT: u32 = 65536; // 64 KB

    pub fn new(is_client: bool) -> Self {
        let initial_id = if is_client { 1 } else { 2 };
        Self {
            is_client,
            next_stream_id: AtomicU32::new(initial_id),
            streams: HashMap::new(),
            initial_credit: Self::DEFAULT_INITIAL_CREDIT,
        }
    }

    pub fn open_stream(&mut self, method_id: u32) -> u32 {
        let id = self.next_stream_id.fetch_add(2, Ordering::SeqCst);
        let status = StreamStatus {
            stream_id: id,
            state: StreamState::Open,
            method_id,
            inbound_credit: self.initial_credit,
            outbound_credit: self.initial_credit,
        };
        self.streams.insert(id, status);
        id
    }

    pub fn register_inbound_stream(&mut self, stream_id: u32, method_id: u32) -> Result<()> {
        if self.streams.contains_key(&stream_id) {
            return Err(SynapseError::MultiplexerError {
                message: format!("Stream ID {} already registered", stream_id),
            });
        }
        let status = StreamStatus {
            stream_id,
            state: StreamState::Open,
            method_id,
            inbound_credit: self.initial_credit,
            outbound_credit: self.initial_credit,
        };
        self.streams.insert(stream_id, status);
        Ok(())
    }

    pub fn handle_inbound_frame(&mut self, frame: &Frame) -> Result<Option<Frame>> {
        let stream_id = frame.header.stream_id;

        // Check control frames that apply to stream 0
        if stream_id == 0 {
            match frame.header.frame_type {
                FrameType::Ping => {
                    let pong = Frame::new(FrameType::Pong, 0, frame.header.method_id, frame.payload.clone());
                    return Ok(Some(pong));
                }
                FrameType::Pong => return Ok(None),
                _ => {}
            }
        }

        // Handle credit updates
        if frame.header.frame_type == FrameType::Credit {
            if let Some(stream) = self.streams.get_mut(&stream_id) {
                if frame.payload.len() >= 4 {
                    let credit = byteorder::BigEndian::read_u32(&frame.payload[0..4]);
                    stream.outbound_credit = stream.outbound_credit.saturating_add(credit);
                }
            }
            return Ok(None);
        }

        // Handle stream cancel
        if frame.header.frame_type == FrameType::StreamCancel {
            if let Some(stream) = self.streams.get_mut(&stream_id) {
                stream.state = StreamState::Closed;
            }
            return Ok(None);
        }

        // Update stream state on end stream flag
        if frame.header.is_end_stream() || frame.header.frame_type == FrameType::StreamClose {
            if let Some(stream) = self.streams.get_mut(&stream_id) {
                match stream.state {
                    StreamState::Open => {
                        stream.state = StreamState::HalfClosedRemote;
                    }
                    StreamState::HalfClosedLocal => {
                        stream.state = StreamState::Closed;
                    }
                    _ => {}
                }
            }
        }

        // Deduct inbound credit
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            let payload_len = frame.payload.len() as u32;
            stream.inbound_credit = stream.inbound_credit.saturating_sub(payload_len);

            // Generate credit replenishment if credit is low
            if stream.inbound_credit < self.initial_credit / 2 {
                let replenishment = self.initial_credit - stream.inbound_credit;
                stream.inbound_credit = self.initial_credit;
                let mut payload = vec![0u8; 4];
                byteorder::BigEndian::write_u32(&mut payload, replenishment);
                let credit_frame = Frame::new(FrameType::Credit, stream_id, 0, payload);
                return Ok(Some(credit_frame));
            }
        }

        Ok(None)
    }

    pub fn close_stream_local(&mut self, stream_id: u32) {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            match stream.state {
                StreamState::Open => {
                    stream.state = StreamState::HalfClosedLocal;
                }
                StreamState::HalfClosedRemote => {
                    stream.state = StreamState::Closed;
                }
                _ => {}
            }
        }
    }

    pub fn get_stream(&self, stream_id: u32) -> Option<&StreamStatus> {
        self.streams.get(&stream_id)
    }

    pub fn active_stream_count(&self) -> usize {
        self.streams.values().filter(|s| s.state != StreamState::Closed).count()
    }
}
