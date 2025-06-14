#![doc = include_str!("../README.md")]

use anyhow::Context;
use clap::Parser;
use std::io::Read;
use std::process::ExitCode;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() -> ExitCode {
    let Err(err) = try_main() else {
        return ExitCode::SUCCESS;
    };

    eprintln!("Error: {err:?}");

    ExitCode::FAILURE
}

fn try_main() -> Result {
    let cli = Cli::parse();

    let input = if cli.input == "-" {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .with_context(|| "Failed to read from stdin")?;
        input
    } else {
        std::fs::read_to_string(&cli.input)
            .with_context(|| format!("Failed to read file '{}'", cli.input))?
    };

    let output = decondenser::Decondenser::generic().decondense(&input)?;

    if cli.output == "-" {
        println!("{output}");
        return Ok(());
    }

    std::fs::write(&cli.output, output)
        .with_context(|| format!("Failed to write to file '{}'", cli.output))
}

/// Pretty-print any text based on brackets nesting
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify a file path or "-" for stdin
    #[clap(long, default_value = "-")]
    input: String,

    /// Specify a file path or "-" for stdout
    #[clap(long, default_value = "-")]
    output: String,

    /// Indentation string to use for pretty-printing
    #[clap(long, default_value = "    ")]
    indent: String,
}
