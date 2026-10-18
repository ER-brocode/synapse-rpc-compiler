use std::path::PathBuf;
use crate::error::Result;
use crate::parser::ast::Schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLanguage {
    Rust,
    TypeScript,
    Go,
}

impl TargetLanguage {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Some(TargetLanguage::Rust),
            "typescript" | "ts" => Some(TargetLanguage::TypeScript),
            "go" | "golang" => Some(TargetLanguage::Go),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            TargetLanguage::Rust => "rs",
            TargetLanguage::TypeScript => "ts",
            TargetLanguage::Go => "go",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    pub target: TargetLanguage,
    pub emit_client: bool,
    pub emit_server: bool,
    pub package_name: Option<String>,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            target: TargetLanguage::Rust,
            emit_client: true,
            emit_server: true,
            package_name: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub relative_path: PathBuf,
    pub content: String,
}

pub trait CodeGenerator {
    fn generate(&self, schema: &Schema, options: &GeneratorOptions) -> Result<Vec<GeneratedFile>>;
}
