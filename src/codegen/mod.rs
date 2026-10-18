pub mod generator;
pub mod golang;
pub mod rust;
pub mod typescript;

pub use generator::{CodeGenerator, GeneratedFile, GeneratorOptions, TargetLanguage};
pub use golang::GoGenerator;
pub use rust::RustGenerator;
pub use typescript::TypeScriptGenerator;

use std::fs;
use std::path::Path;
use crate::error::Result;
use crate::parser::ast::Schema;

pub fn generate_for_target(
    schema: &Schema,
    target: TargetLanguage,
    options: &GeneratorOptions,
) -> Result<Vec<GeneratedFile>> {
    match target {
        TargetLanguage::Rust => RustGenerator::new().generate(schema, options),
        TargetLanguage::TypeScript => TypeScriptGenerator::new().generate(schema, options),
        TargetLanguage::Go => GoGenerator::new().generate(schema, options),
    }
}

pub fn write_generated_files(files: &[GeneratedFile], output_dir: &Path) -> Result<()> {
    for file in files {
        let dest = output_dir.join(&file.relative_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, &file.content)?;
    }
    Ok(())
}
