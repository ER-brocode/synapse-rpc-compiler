use synapse_compiler::runtime::{
    Frame, FrameType, FLAG_END_STREAM, HEADER_SIZE, SYNAPSE_MAGIC, SYNAPSE_VERSION,
};

#[test]
fn test_frame_encode_decode_roundtrip() {
    let payload = b"Hello, Synapse RPC protocol!".to_vec();
    let frame = Frame::new(FrameType::CallRequest, 101, 5, payload.clone())
        .with_flags(FLAG_END_STREAM);

    let bytes = frame.encode();
    assert_eq!(bytes.len(), HEADER_SIZE + payload.len());

    let decoded = Frame::decode(&bytes).expect("Frame should decode successfully");
    assert_eq!(decoded.header.magic, SYNAPSE_MAGIC);
    assert_eq!(decoded.header.version, SYNAPSE_VERSION);
    assert_eq!(decoded.header.frame_type, FrameType::CallRequest);
    assert_eq!(decoded.header.stream_id, 101);
    assert_eq!(decoded.header.method_id, 5);
    assert!(decoded.header.is_end_stream());
    assert_eq!(decoded.payload, payload);
}

#[test]
fn test_corrupted_crc32_rejected() {
    let payload = b"critical telemetry payload".to_vec();
    let frame = Frame::new(FrameType::StreamData, 42, 1, payload);
    let mut bytes = frame.encode();

    // Corrupt one payload byte
    bytes[HEADER_SIZE + 2] ^= 0xFF;

    let res = Frame::decode(&bytes);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("CRC32 checksum mismatch"));
}
