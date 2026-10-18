use synapse_compiler::error::Result;
use synapse_compiler::runtime::{
    SynapseClient, SynapseCodec, SynapseDecoder, SynapseEncoder, SynapseServer,
};

#[derive(Debug, Clone, PartialEq)]
struct EchoRequest {
    message: String,
    counter: u32,
}

impl SynapseCodec for EchoRequest {
    fn encode(&self, encoder: &mut SynapseEncoder) -> Result<()> {
        encoder.write_string(1, &self.message);
        encoder.write_u32(2, self.counter);
        Ok(())
    }

    fn decode(decoder: &mut SynapseDecoder) -> Result<Self> {
        let mut message = String::new();
        let mut counter = 0;
        while let Some((id, wire_type)) = decoder.read_tag()? {
            match id {
                1 => message = decoder.read_string()?,
                2 => counter = decoder.read_u32()?,
                _ => decoder.skip_field(wire_type)?,
            }
        }
        Ok(Self { message, counter })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct EchoResponse {
    reply: String,
    ack: bool,
}

impl SynapseCodec for EchoResponse {
    fn encode(&self, encoder: &mut SynapseEncoder) -> Result<()> {
        encoder.write_string(1, &self.reply);
        encoder.write_bool(2, self.ack);
        Ok(())
    }

    fn decode(decoder: &mut SynapseDecoder) -> Result<Self> {
        let mut reply = String::new();
        let mut ack = false;
        while let Some((id, wire_type)) = decoder.read_tag()? {
            match id {
                1 => reply = decoder.read_string()?,
                2 => ack = decoder.read_bool()?,
                _ => decoder.skip_field(wire_type)?,
            }
        }
        Ok(Self { reply, ack })
    }
}

#[test]
fn test_end_to_end_rpc_cycle() {
    let mut server = SynapseServer::new();
    let echo_method_id = 1001;

    // Register typed RPC handler
    server.register_typed_handler(echo_method_id, |req: EchoRequest| -> Result<EchoResponse> {
        Ok(EchoResponse {
            reply: format!("ECHO: {} (#{})", req.message, req.counter),
            ack: true,
        })
    });

    let mut client = SynapseClient::new();
    let request = EchoRequest {
        message: "Hello Synapse!".to_string(),
        counter: 1,
    };

    // Client creates unary request frame
    let (stream_id, request_frame) = client.create_unary_request(echo_method_id, &request)
        .expect("Client request creation failed");

    // Wire transit (simulate network)
    let wire_bytes = request_frame.encode();

    // Server receives and decodes frame
    let incoming_frame = synapse_compiler::runtime::Frame::decode(&wire_bytes)
        .expect("Frame decoding failed on wire");

    let response_frame = server.handle_frame(&incoming_frame)
        .expect("Server handler failed");

    // Client parses response frame
    let response: EchoResponse = client.parse_unary_response(&response_frame)
        .expect("Client parsing failed");

    assert_eq!(response.reply, "ECHO: Hello Synapse! (#1)");
    assert!(response.ack);
}
