use synapse_compiler::parser::{parse_and_validate, Lexer, Parser, StreamingMode};

#[test]
fn test_parse_minimal_schema() {
    let idl = r#"
        namespace test.minimal;

        enum Status {
            Active = 0,
            Suspended = 1,
        }

        struct User {
            1: u64 id;
            2: string name;
            3: Status status;
        }

        service UserService {
            rpc GetUser(User) -> User;
        }
    "#;

    let schema = parse_and_validate(idl).expect("Schema should parse successfully");
    assert_eq!(schema.namespace.unwrap().to_string(), "test.minimal");
    assert_eq!(schema.enums.len(), 1);
    assert_eq!(schema.structs.len(), 1);
    assert_eq!(schema.services.len(), 1);

    let user_struct = &schema.structs[0];
    assert_eq!(user_struct.name, "User");
    assert_eq!(user_struct.fields.len(), 3);
    assert_eq!(user_struct.fields[0].id, 1);
    assert_eq!(user_struct.fields[0].name, "id");

    let user_service = &schema.services[0];
    assert_eq!(user_service.name, "UserService");
    assert_eq!(user_service.methods.len(), 1);
    assert_eq!(user_service.methods[0].name, "GetUser");
    assert_eq!(user_service.methods[0].streaming_mode, StreamingMode::Unary);
}

#[test]
fn test_parse_streaming_methods() {
    let idl = r#"
        namespace test.streaming;

        struct DataChunk {
            1: u32 sequence;
            2: bytes payload;
        }

        struct StreamAck {
            1: bool success;
        }

        service StreamService {
            rpc Download(DataChunk) -> stream DataChunk;
            rpc Upload(stream DataChunk) -> StreamAck;
            rpc Duplex(stream DataChunk) -> stream DataChunk;
            subscribe OnEvent(DataChunk) -> stream DataChunk;
        }
    "#;

    let schema = parse_and_validate(idl).expect("Streaming schema should parse");
    let svc = &schema.services[0];
    assert_eq!(svc.methods.len(), 4);
    assert_eq!(svc.methods[0].streaming_mode, StreamingMode::ServerStreaming);
    assert_eq!(svc.methods[1].streaming_mode, StreamingMode::ClientStreaming);
    assert_eq!(svc.methods[2].streaming_mode, StreamingMode::BiDirectional);
    assert_eq!(svc.methods[3].streaming_mode, StreamingMode::ServerStreaming);
}

#[test]
fn test_parse_doc_comments_and_attributes() {
    let idl = r#"
        namespace test.docs;

        /// High-priority user message
        @priority(1)
        struct Alert {
            /// Unique identifier
            1: u64 id;
            /// Description text
            2: string message;
        }
    "#;

    let schema = parse_and_validate(idl).expect("Doc comments should parse");
    let alert = &schema.structs[0];
    assert_eq!(alert.doc.as_deref(), Some("High-priority user message"));
    assert_eq!(alert.attributes.len(), 1);
    assert_eq!(alert.attributes[0].name, "priority");
    assert_eq!(alert.fields[0].doc.as_deref(), Some("Unique identifier"));
}
