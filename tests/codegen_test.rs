use synapse_compiler::codegen::{generate_for_target, GeneratorOptions, TargetLanguage};
use synapse_compiler::parser::parse_and_validate;

const SAMPLE_SCHEMA: &str = r#"
namespace synapse.chat.v1;

enum UserStatus {
    Offline = 0,
    Online = 1,
    Busy = 2,
}

struct ChatMessage {
    1: u64 id;
    2: string sender;
    3: string content;
    4: u64 timestamp;
}

struct MessageReceipt {
    1: u64 message_id;
    2: bool delivered;
}

service ChatService {
    rpc SendMessage(ChatMessage) -> MessageReceipt;
    rpc StreamMessages(ChatMessage) -> stream ChatMessage;
}
"#;

#[test]
fn test_rust_codegen() {
    let schema = parse_and_validate(SAMPLE_SCHEMA).expect("Schema valid");
    let options = GeneratorOptions {
        target: TargetLanguage::Rust,
        emit_client: true,
        emit_server: true,
        package_name: None,
    };

    let files = generate_for_target(&schema, TargetLanguage::Rust, &options).expect("Rust codegen failed");
    assert_eq!(files.len(), 1);
    let content = &files[0].content;

    assert!(content.contains("pub enum UserStatus"));
    assert!(content.contains("pub struct ChatMessage"));
    assert!(content.contains("pub struct MessageReceipt"));
    assert!(content.contains("impl SynapseCodec for ChatMessage"));
    assert!(content.contains("pub trait ChatServiceServer"));
    assert!(content.contains("pub struct ChatServiceClient"));
}

#[test]
fn test_typescript_codegen() {
    let schema = parse_and_validate(SAMPLE_SCHEMA).expect("Schema valid");
    let options = GeneratorOptions {
        target: TargetLanguage::TypeScript,
        emit_client: true,
        emit_server: true,
        package_name: None,
    };

    let files = generate_for_target(&schema, TargetLanguage::TypeScript, &options).expect("TS codegen failed");
    assert_eq!(files.len(), 1);
    let content = &files[0].content;

    assert!(content.contains("export enum UserStatus"));
    assert!(content.contains("export interface ChatMessage"));
    assert!(content.contains("export interface MessageReceipt"));
    assert!(content.contains("export class ChatServiceClient"));
    assert!(content.contains("export class ChatServiceDispatcher"));
    assert!(content.contains("AsyncIterable<ChatMessage>"));
}

#[test]
fn test_golang_codegen() {
    let schema = parse_and_validate(SAMPLE_SCHEMA).expect("Schema valid");
    let options = GeneratorOptions {
        target: TargetLanguage::Go,
        emit_client: true,
        emit_server: true,
        package_name: Some("chatv1".to_string()),
    };

    let files = generate_for_target(&schema, TargetLanguage::Go, &options).expect("Go codegen failed");
    assert_eq!(files.len(), 1);
    let content = &files[0].content;

    assert!(content.contains("package chatv1"));
    assert!(content.contains("type UserStatus int32"));
    assert!(content.contains("type ChatMessage struct"));
    assert!(content.contains("type ChatServiceClient interface"));
    assert!(content.contains("type ChatServiceServer interface"));
}
