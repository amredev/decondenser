#![doc = include_str!("../README.md")]

use anyhow::Context;
use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects, Style};
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

    let output = decondenser::Decondenser::generic()
        .max_line_size(cli.max_line_size)
        .no_break_size(cli.no_break_size)
        .debug_indent(cli.debug_indent)
        .debug_layout(cli.debug_layout)
        .decondense(&input);

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

const CLI_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

#[derive(Parser)]
#[command(
    name = "decondenser",
    about = "Pretty-print any text based on brackets nesting.",
    long_about = None,
    version,
    styles = CLI_STYLES,
)]
struct Cli {
    /// Specify a file path or "-" for stdin
    #[clap(long, default_value = "-")]
    input: String,

    /// Specify a file path or "-" for stdout
    #[clap(long, default_value = "-")]
    output: String,

    /// String to use as a single level of indentation nesting.
    #[clap(long, default_value = "    ")]
    indent: String,

    /// Best-effort max size of a line to fit into.
    #[clap(long, default_value_t = 80)]
    max_line_size: usize,

    /// Lines shorter than this will never be broken up at any indentation level.
    #[clap(long, default_value_t = 40)]
    no_break_size: usize,

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
