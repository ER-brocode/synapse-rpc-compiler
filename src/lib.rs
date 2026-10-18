pub mod codegen;
pub mod error;
pub mod parser;
pub mod runtime;

pub use codegen::{
    generate_for_target, write_generated_files, CodeGenerator, GeneratedFile, GeneratorOptions,
    GoGenerator, RustGenerator, TargetLanguage, TypeScriptGenerator,
};
pub use error::{Result, SynapseError};
pub use parser::{parse_and_validate, Lexer, Parser, Schema, Validator};
pub use runtime::{
    decode_zigzag32, decode_zigzag64, encode_zigzag32, encode_zigzag64, Frame, FrameHeader,
    FrameType, Multiplexer, SynapseClient, SynapseCodec, SynapseDecoder, SynapseEncoder,
    SynapseServer, WireType, FLAG_ACK, FLAG_COMPRESSED, FLAG_END_STREAM, HEADER_SIZE,
    SYNAPSE_MAGIC, SYNAPSE_VERSION,
};

use std::fs;
use std::path::Path;

pub struct SynapseCompiler;

impl SynapseCompiler {
    pub fn compile_file<P: AsRef<Path>>(
        schema_path: P,
        output_dir: P,
        targets: &[TargetLanguage],
    ) -> Result<Vec<GeneratedFile>> {
        let content = fs::read_to_string(schema_path)?;
        Self::compile_str(&content, output_dir, targets)
    }

    pub fn compile_str<P: AsRef<Path>>(
        schema_source: &str,
        output_dir: P,
        targets: &[TargetLanguage],
    ) -> Result<Vec<GeneratedFile>> {
        let schema = parse_and_validate(schema_source)?;
        let mut all_files = Vec::new();

        for target in targets {
            let options = GeneratorOptions {
                target: *target,
                emit_client: true,
                emit_server: true,
                package_name: None,
            };
            let files = generate_for_target(&schema, *target, &options)?;
            write_generated_files(&files, output_dir.as_ref())?;
            all_files.extend(files);
        }

        Ok(all_files)
    }
}
