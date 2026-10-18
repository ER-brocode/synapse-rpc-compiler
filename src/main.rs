use clap::{Parser as ClapParser, Subcommand};
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process;
use synapse_compiler::codegen::TargetLanguage;
use synapse_compiler::error::Result;
use synapse_compiler::parser::parse_and_validate;
use synapse_compiler::SynapseCompiler;

#[derive(ClapParser)]
#[command(name = "synapse-compiler")]
#[command(author = "koushik4475 <koushik4475@gmail.com>")]
#[command(version = "0.2.0")]
#[command(about = "Full-Stack Reactive Schema & Type-Safe RPC Compiler in Rust and TypeScript", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Synapse IDL schema to target languages
    Compile {
        /// Path to the .synapse schema file
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for generated code
        #[arg(short, long, default_value = "./generated")]
        out_dir: PathBuf,

        /// Target languages (comma-separated: rust, typescript, go)
        #[arg(short, long, default_value = "rust,typescript,go")]
        targets: String,
    },

    /// Validate a Synapse IDL schema without code generation
    Validate {
        /// Path to the .synapse schema file
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Inspect and print the parsed AST of a schema
    Inspect {
        /// Path to the .synapse schema file
        #[arg(short, long)]
        input: PathBuf,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, out_dir, targets } => {
            println!("{}", "⚡ Compiling Synapse Schema...".cyan().bold());
            let target_list: Vec<TargetLanguage> = targets
                .split(',')
                .filter_map(|t| TargetLanguage::from_str(t.trim()))
                .collect();

            if target_list.is_empty() {
                eprintln!("{}", "Error: No valid target languages specified.".red().bold());
                process::exit(1);
            }

            match SynapseCompiler::compile_file(&input, &out_dir, &target_list) {
                Ok(files) => {
                    println!("{}", format!("✔ Successfully generated {} files into {}:", files.len(), out_dir.display()).green().bold());
                    for file in &files {
                        println!("  {} {}", "•".green(), file.relative_path.display());
                    }
                }
                Err(err) => {
                    eprintln!("{} {}", "✖ Compilation failed:".red().bold(), err);
                    process::exit(1);
                }
            }
        }
        Commands::Validate { input } => {
            println!("{}", format!("⚡ Validating schema: {}", input.display()).cyan());
            match fs::read_to_string(&input) {
                Ok(content) => match parse_and_validate(&content) {
                    Ok(schema) => {
                        println!("{}", "✔ Schema is valid!".green().bold());
                        println!("  Structs:  {}", schema.structs.len());
                        println!("  Enums:    {}", schema.enums.len());
                        println!("  Services: {}", schema.services.len());
                    }
                    Err(err) => {
                        eprintln!("{} {}", "✖ Validation error:".red().bold(), err);
                        process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("{} {}", "✖ Failed to read input file:".red().bold(), err);
                    process::exit(1);
                }
            }
        }
        Commands::Inspect { input, json } => {
            match fs::read_to_string(&input) {
                Ok(content) => match parse_and_validate(&content) {
                    Ok(schema) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
                        } else {
                            println!("{}", "Schema Structure:".cyan().bold());
                            if let Some(ns) = schema.namespace {
                                println!("  Namespace: {}", ns.to_string());
                            }
                            for s in &schema.structs {
                                println!("  Struct: {} ({} fields)", s.name.yellow(), s.fields.len());
                            }
                            for e in &schema.enums {
                                println!("  Enum: {} ({} variants)", e.name.magenta(), e.variants.len());
                            }
                            for svc in &schema.services {
                                println!("  Service: {} ({} methods)", svc.name.blue().bold(), svc.methods.len());
                                for m in &svc.methods {
                                    println!("    - {} ({:?})", m.name, m.streaming_mode);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("{} {}", "✖ Parse error:".red().bold(), err);
                        process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("{} {}", "✖ Failed to read file:".red().bold(), err);
                    process::exit(1);
                }
            }
        }
    }
}
