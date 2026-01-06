//! hurl-adapt CLI - Convert KDL files to Hurl format

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use kdl::KdlDocument;
use thiserror::Error;

use hurl_adapters_lib::formats::kdl::{TranslationError, translate_to_string};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Error)]
enum CliError {
    #[error("failed to read file '{path}'")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to read from stdin")]
    ReadStdin(#[source] io::Error),

    #[error("failed to write file '{path}'")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write to stdout")]
    WriteStdout(#[source] io::Error),

    #[error("KDL parse error: {0}")]
    KdlParse(#[source] kdl::KdlError),

    #[error("translation error: {0}")]
    Translation(#[source] TranslationError),
}

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

fn read_input(path: Option<&PathBuf>) -> Result<String, CliError> {
    if let Some(p) = path {
        fs::read_to_string(p).map_err(|e| CliError::ReadFile {
            path: p.clone(),
            source: e,
        })
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(CliError::ReadStdin)?;
        Ok(buffer)
    }
}

fn write_output(path: Option<&PathBuf>, content: &str) -> Result<(), CliError> {
    if let Some(p) = path {
        fs::write(p, content).map_err(|e| CliError::WriteFile {
            path: p.clone(),
            source: e,
        })
    } else {
        io::stdout()
            .write_all(content.as_bytes())
            .map_err(CliError::WriteStdout)
    }
}

fn run(args: &Args) -> Result<(), CliError> {
    // Read input (file or stdin)
    let input = read_input(args.input.as_ref())?;

    // Parse KDL document
    let doc: KdlDocument = input.parse().map_err(CliError::KdlParse)?;

    // Translate to Hurl format (format arg is for future extensibility)
    let hurl_output = match args.format {
        InputFormat::Kdl => translate_to_string(&doc).map_err(CliError::Translation)?,
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
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
