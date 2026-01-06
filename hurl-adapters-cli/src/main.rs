//! hurl-adapt CLI - Convert KDL files to Hurl format

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use kdl::KdlDocument;

use hurl_adapters_lib::formats::kdl::translate_to_string;

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum InputFormat {
    #[default]
    Kdl,
}

/// Convert KDL files to Hurl format for HTTP testing
#[derive(Parser, Debug)]
#[command(name = "hurl-adapt")]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file path (reads from stdin if not provided)
    input: Option<PathBuf>,

    /// Input format
    #[arg(short, long, value_enum, default_value_t = InputFormat::Kdl)]
    format: InputFormat,

    /// Output file (writes to stdout if not provided)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Validate input without producing output
    #[arg(short, long)]
    check: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    quiet: bool,
}

// ============================================================================
// Core Logic
// ============================================================================

fn read_input(path: Option<&PathBuf>) -> Result<String> {
    if let Some(p) = path {
        fs::read_to_string(p).with_context(|| format!("failed to read file '{}'", p.display()))
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("failed to read from stdin")?;
        Ok(buffer)
    }
}

fn write_output(path: Option<&PathBuf>, content: &str) -> Result<()> {
    if let Some(p) = path {
        fs::write(p, content).with_context(|| format!("failed to write file '{}'", p.display()))
    } else {
        io::stdout()
            .write_all(content.as_bytes())
            .context("failed to write to stdout")
    }
}

fn run(args: &Args) -> Result<()> {
    // Read input (file or stdin)
    let input = read_input(args.input.as_ref())?;

    // Parse KDL document
    let doc: KdlDocument = input.parse().context("KDL parse error")?;

    // Translate to Hurl format (format arg is for future extensibility)
    let hurl_output = match args.format {
        InputFormat::Kdl => translate_to_string(&doc).context("translation error")?,
    };

    // Check mode: validate only, no output
    if args.check {
        if !args.quiet {
            let source = args
                .input
                .as_ref()
                .map_or_else(|| "stdin".to_string(), |p| p.display().to_string());
            eprintln!("{source}: OK");
        }
        return Ok(());
    }

    // Write output (file or stdout)
    write_output(args.output.as_ref(), &hurl_output)?;

    Ok(())
}

// ============================================================================
// Entry Point
// ============================================================================

fn main() -> ExitCode {
    let args = Args::parse();

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
