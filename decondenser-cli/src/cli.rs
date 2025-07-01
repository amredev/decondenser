use crate::config::Config;
use anyhow::Context;
use clap::Parser;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "decondenser",
    about =
        "Pretty-print any text based on brackets nesting. \
        More docs: https://decondenser.dev",
    long_about = None,
    version,
    styles = crate::styling::CLI_STYLES,
)]
pub(crate) struct Cli {
    /// Specify a file path or "-" for stdin
    #[clap(long, default_value = "-")]
    input: String,

    /// Specify a file path or "-" for stdout
    #[clap(long, default_value = "-")]
    output: String,

    /// Language profile to use. Can be either a built-in language or a custom
    /// one defined in the config file.
    #[clap(long, default_value = "generic")]
    lang: String,

    /// Path to the config file. By default, will search for a file named
    /// "decondenser.toml" in the current and parent directories.
    #[clap(long)]
    config: Option<PathBuf>,

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
    debug_line_size: bool,
}

impl Cli {
    pub(crate) fn run(self) -> crate::Result {
        let input = if self.input == "-" {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .with_context(|| "Failed to read from stdin")?;
            input
        } else {
            std::fs::read_to_string(&self.input)
                .with_context(|| format!("Failed to read file '{}'", self.input))?
        };

        let config = match self.config {
            Some(config) => Config::from_file(&config)?.with_context(|| {
                format!(
                    "Config file was not found at the specified path: '{}'",
                    config.display()
                )
            })?,
            None => Config::discover()?.unwrap_or_default(),
        };

        let output = config
            .into_decondenser(&self.lang)?
            .debug_indent(self.debug_indent)
            .debug_layout(self.debug_layout)
            .decondense(&input);

        if self.debug_line_size {
            let max_width = output
                .lines()
                // TODO: use `unicode-width` crate
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0);

            eprintln!("Max line width: {max_width}");
        }

        if self.output == "-" {
            println!("{output}");
            return Ok(());
        }

        std::fs::write(&self.output, output)
            .with_context(|| format!("Failed to write to file '{}'", self.output))
    }
}
