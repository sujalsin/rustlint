mod config;
mod linter;
mod rules;
mod processor;

use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use colored::*;
use crate::rules::Rule;
use env_logger;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Files or directories to lint")]
    paths: Vec<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    
    if args.paths.is_empty() {
        println!("No files specified. Use --help for usage information.");
        return Ok(());
    }

    let rules: Vec<Box<dyn Rule + Sync>> = rules::get_default_rules()
        .into_iter()
        .map(|r| r as Box<dyn Rule + Sync>)
        .collect();

    let mut all_files = Vec::new();

    // Collect all Python files from the specified paths
    for path in args.paths {
        if path.is_dir() {
            let mut python_files = processor::find_python_files(&path)?;
            all_files.append(&mut python_files);
        } else if path.is_file() {
            all_files.push(path);
        }
    }

    // Process files in parallel
    let diagnostics = processor::process_files(all_files, &rules)?;

    // Print diagnostics
    for diagnostic in diagnostics {
        let level_str = match diagnostic.level {
            linter::DiagnosticLevel::Error => "error".red(),
            linter::DiagnosticLevel::Warning => "warning".yellow(),
        };

        println!(
            "{}: {} at {}:{}",
            level_str,
            diagnostic.message,
            diagnostic.path,
            diagnostic.line
        );
    }

    Ok(())
}
