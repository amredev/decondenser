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

    let mut decondenser = decondenser::Decondenser::generic();
    decondenser.line_size = cli.line_size;
    decondenser.debug_indent = cli.debug_indent;
    decondenser.debug_layout = cli.debug_layout;

    let output = decondenser.decondense(&input);

    if cli.debug_line_width {
        let max_width = output
            .lines()
            // TODO: use `unicode-width` crate
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);

        eprintln!("Max line width: {max_width}");
    }

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

    /// Desired size of a line. If the is too big to fit into a single line of
    /// this size, it'll be broken into several lines. If the content is too
    /// small so that it doesn't fill the entire line, then several lines can be
    /// condensed into a single line.
    ///
    /// There is no guarantee that the output will not contain lines longer than
    /// this size. For example, a single long string literal or a long sequence
    /// of non-whitespace characters may span more than this many characters,
    /// and decondenser does not currently attempt to break these up.
    #[clap(long, default_value_t = 80)]
    line_size: usize,

    /// Only used for debugging by the decondenser developers.
    ///
    /// Enables outputting of the layout control characters.
    #[clap(long, hide = true)]
    debug_layout: bool,

    /// Only used for debugging by the decondenser developers.
    ///
    /// Enables outputting of the indent control characters.
    #[clap(long, hide = true)]
    debug_indent: bool,

    /// Show the line width in the output.
    #[clap(
        long,
        hide = true,
        conflicts_with = "debug_layout",
        conflicts_with = "debug_indent"
    )]
    debug_line_width: bool,
}
