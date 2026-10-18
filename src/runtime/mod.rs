pub mod client;
pub mod codec;
pub mod multiplexer;
pub mod protocol;
pub mod server;

pub use client::SynapseClient;
pub use codec::{
    decode_zigzag32, decode_zigzag64, encode_zigzag32, encode_zigzag64,
    SynapseCodec, SynapseDecoder, SynapseEncoder, WireType,
};
pub use multiplexer::{Multiplexer, StreamState, StreamStatus};
pub use protocol::{
    Frame, FrameHeader, FrameType, FLAG_ACK, FLAG_COMPRESSED, FLAG_END_STREAM,
    HEADER_SIZE, SYNAPSE_MAGIC, SYNAPSE_VERSION,
};
pub use server::SynapseServer;
