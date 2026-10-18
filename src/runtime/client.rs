use crate::error::{Result, SynapseError};
use crate::runtime::codec::{SynapseCodec, SynapseDecoder, SynapseEncoder};
use crate::runtime::multiplexer::Multiplexer;
use crate::runtime::protocol::{Frame, FrameType, FLAG_END_STREAM};

pub struct SynapseClient {
    multiplexer: Multiplexer,
}

impl SynapseClient {
    pub fn new() -> Self {
        Self {
            multiplexer: Multiplexer::new(true),
        }
    }

    pub fn create_unary_request<Req: SynapseCodec>(&mut self, method_id: u32, req: &Req) -> Result<(u32, Frame)> {
        let stream_id = self.multiplexer.open_stream(method_id);
        let payload = req.to_bytes()?;
        let frame = Frame::new(FrameType::CallRequest, stream_id, method_id, payload)
            .with_flags(FLAG_END_STREAM);
        self.multiplexer.close_stream_local(stream_id);
        Ok((stream_id, frame))
    }

    pub fn parse_unary_response<Resp: SynapseCodec>(&mut self, frame: &Frame) -> Result<Resp> {
        if frame.header.frame_type == FrameType::Error {
            let error_msg = String::from_utf8_lossy(&frame.payload).to_string();
            return Err(SynapseError::ProtocolError {
                message: format!("Server returned error: {}", error_msg),
            });
        }

        if frame.header.frame_type != FrameType::CallResponse {
            return Err(SynapseError::ProtocolError {
                message: format!("Expected CallResponse frame, got {:?}", frame.header.frame_type),
            });
        }

        self.multiplexer.handle_inbound_frame(frame)?;
        Resp::from_bytes(&frame.payload)
    }

    pub fn create_stream_data<Item: SynapseCodec>(&mut self, stream_id: u32, method_id: u32, item: &Item, is_last: bool) -> Result<Frame> {
        let payload = item.to_bytes()?;
        let mut frame = Frame::new(FrameType::StreamData, stream_id, method_id, payload);
        if is_last {
            frame = frame.with_flags(FLAG_END_STREAM);
            self.multiplexer.close_stream_local(stream_id);
        }
        Ok(frame)
    }

    pub fn parse_stream_data<Item: SynapseCodec>(&mut self, frame: &Frame) -> Result<(Item, bool)> {
        if frame.header.frame_type == FrameType::Error {
            let error_msg = String::from_utf8_lossy(&frame.payload).to_string();
            return Err(SynapseError::ProtocolError {
                message: format!("Server returned streaming error: {}", error_msg),
            });
        }

        let is_end = frame.header.is_end_stream();
        self.multiplexer.handle_inbound_frame(frame)?;
        let item = Item::from_bytes(&frame.payload)?;
        Ok((item, is_end))
    }
}
