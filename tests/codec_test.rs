use synapse_compiler::runtime::{
    decode_zigzag32, decode_zigzag64, encode_zigzag32, encode_zigzag64,
    SynapseDecoder, SynapseEncoder, WireType,
};

#[test]
fn test_varint_u32_roundtrip() {
    let test_values = [0u32, 1, 127, 128, 255, 300, 16384, 65535, 1_000_000, u32::MAX];
    for &val in &test_values {
        let mut encoder = SynapseEncoder::new();
        encoder.write_varint_u32(val);

        let mut decoder = SynapseDecoder::new(encoder.as_slice());
        let decoded = decoder.read_varint_u32().expect("Failed to read varint");
        assert_eq!(decoded, val, "Mismatch for value {}", val);
        assert!(decoder.is_eof());
    }
}

#[test]
fn test_zigzag_roundtrip() {
    let test_i32 = [0, -1, 1, -2, 2, i32::MIN, i32::MAX, -123456, 123456];
    for &v in &test_i32 {
        let encoded = encode_zigzag32(v);
        let decoded = decode_zigzag32(encoded);
        assert_eq!(decoded, v);
    }

    let test_i64 = [0i64, -1, 1, -1000, 1000, i64::MIN, i64::MAX];
    for &v in &test_i64 {
        let encoded = encode_zigzag64(v);
        let decoded = decode_zigzag64(encoded);
        assert_eq!(decoded, v);
    }
}

#[test]
fn test_encoder_decoder_primitives() {
    let mut encoder = SynapseEncoder::new();
    encoder.write_bool(1, true);
    encoder.write_u32(2, 42);
    encoder.write_i64(3, -999999);
    encoder.write_f64(4, 3.1415926535);
    encoder.write_string(5, "Synapse RPC");
    encoder.write_bytes(6, &[0xDE, 0xAD, 0xBE, 0xEF]);

    let bytes = encoder.into_bytes();
    let mut decoder = SynapseDecoder::new(&bytes);

    let mut found_bool = false;
    let mut found_u32 = 0;
    let mut found_i64 = 0;
    let mut found_f64 = 0.0;
    let mut found_string = String::new();
    let mut found_bytes = Vec::new();

    while let Some((id, wire_type)) = decoder.read_tag().unwrap() {
        match id {
            1 => found_bool = decoder.read_bool().unwrap(),
            2 => found_u32 = decoder.read_u32().unwrap(),
            3 => found_i64 = decoder.read_i64().unwrap(),
            4 => found_f64 = decoder.read_f64().unwrap(),
            5 => found_string = decoder.read_string().unwrap(),
            6 => found_bytes = decoder.read_bytes().unwrap(),
            _ => decoder.skip_field(wire_type).unwrap(),
        }
    }

    assert!(found_bool);
    assert_eq!(found_u32, 42);
    assert_eq!(found_i64, -999999);
    assert!((found_f64 - 3.1415926535).abs() < 1e-6);
    assert_eq!(found_string, "Synapse RPC");
    assert_eq!(found_bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
}
