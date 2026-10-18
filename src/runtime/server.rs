use std::collections::HashMap;
use crate::error::{Result, SynapseError};
use crate::runtime::codec::{SynapseCodec, SynapseDecoder, SynapseEncoder};
use crate::runtime::multiplexer::Multiplexer;
use crate::runtime::protocol::{Frame, FrameType, FLAG_END_STREAM};

pub type MethodHandler = Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync>;

pub struct SynapseServer {
    multiplexer: Multiplexer,
    handlers: HashMap<u32, MethodHandler>,
}

impl SynapseServer {
    pub fn new() -> Self {
        Self {
            multiplexer: Multiplexer::new(false),
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<F>(&mut self, method_id: u32, handler: F)
    where
        F: Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync + 'static,
    {
        self.handlers.insert(method_id, Box::new(handler));
    }

    pub fn register_typed_handler<Req, Resp, F>(&mut self, method_id: u32, handler: F)
    where
        Req: SynapseCodec + 'static,
        Resp: SynapseCodec + 'static,
        F: Fn(Req) -> Result<Resp> + Send + Sync + 'static,
    {
        let boxed = Box::new(move |bytes: &[u8]| -> Result<Vec<u8>> {
            let req = Req::from_bytes(bytes)?;
            let resp = handler(req)?;
            resp.to_bytes()
        });
        self.handlers.insert(method_id, boxed);
    }

    pub fn handle_frame(&mut self, frame: &Frame) -> Result<Frame> {
        let stream_id = frame.header.stream_id;
        let method_id = frame.header.method_id;

        self.multiplexer.register_inbound_stream(stream_id, method_id)?;
        self.multiplexer.handle_inbound_frame(frame)?;

        match frame.header.frame_type {
            FrameType::CallRequest => {
                if let Some(handler) = self.handlers.get(&method_id) {
                    match handler(&frame.payload) {
                        Ok(resp_payload) => {
                            let response_frame = Frame::new(
                                FrameType::CallResponse,
                                stream_id,
                                method_id,
                                resp_payload,
                            ).with_flags(FLAG_END_STREAM);
                            self.multiplexer.close_stream_local(stream_id);
                            Ok(response_frame)
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            let err_frame = Frame::new(
                                FrameType::Error,
                                stream_id,
                                method_id,
                                err_msg.into_bytes(),
                            ).with_flags(FLAG_END_STREAM);
                            self.multiplexer.close_stream_local(stream_id);
                            Ok(err_frame)
                        }
                    }
                } else {
                    let err_msg = format!("Method ID {} not found", method_id);
                    let err_frame = Frame::new(
                        FrameType::Error,
                        stream_id,
                        method_id,
                        err_msg.into_bytes(),
                    ).with_flags(FLAG_END_STREAM);
                    self.multiplexer.close_stream_local(stream_id);
                    Ok(err_frame)
                }
            }
            FrameType::Ping => {
                let pong = Frame::new(FrameType::Pong, stream_id, method_id, frame.payload.clone());
                Ok(pong)
            }
            _ => {
                Err(SynapseError::ProtocolError {
                    message: format!("Unhandled frame type in server: {:?}", frame.header.frame_type),
                })
            }
        }
    }
}
