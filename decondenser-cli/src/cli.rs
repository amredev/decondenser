use crate::config::Config;
use crate::{Files, Result};
use anyhow::Context;
use clap::{arg, value_parser};
use std::io::Read;
use std::path::{Path, PathBuf};

fn cli() -> clap::Command {
    clap::Command::new("decondenser")
        .about(
            "decondenser pretty-prints any text based on brackets nesting\n\n\
            this help is short, more details here: https://decondenser.dev",
        )
        .long_about(None)
        .version(env!("CARGO_PKG_VERSION"))
        .styles(crate::styles::CLI_STYLES)
        .next_help_heading("Main")
        .arg(arg!(--input <INPUT> "file path or - for stdin").default_value("-"))
        .arg(arg!(--output <OUTPUT> "file path or - for stdout").default_value("-"))
        .arg(
            arg!(
                --config <CONFIG>
                "path to the config file [default: decondenser.yml in this or parent directories]"
            )
            .value_parser(value_parser!(PathBuf))
            .required(false),
        )
        .next_help_heading("Overrides")
        .arg(arg!(--indent <INDENT> "a number of spaces or a string to use for indentation [default: 4]"))
        .arg(arg!(
            --"max-line-size" <SIZE>
            "best-effort max size of a line to fit into; see also --no-break-size \
            [default: 80]"
        ))
        .arg(arg!(
            --"no-break-size" <SIZE>
            "lines shorter than this (ignoring indent) won't be broken \
            [default: --max-line-size / 2]"
        ))
        .next_help_heading("Meta")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(arg!(-h --help "Print help").action(clap::ArgAction::Help))
        .arg(arg!(-V --version "Print version").action(clap::ArgAction::Version))
}

pub(crate) fn run(files: &mut Files) -> Result {
    let mut matches = cli().get_matches();

    let input = matches.remove_one::<String>("input").unwrap();
    let output = matches.remove_one::<String>("output").unwrap();
    let config = matches.remove_one::<PathBuf>("config");
    let indent = matches.remove_one::<String>("indent");
    let max_line_size = matches.remove_one::<usize>("max-line-size");
    let no_break_size = matches.remove_one::<usize>("no-break-size");

    let config = config_or_default(config.as_deref(), files)?;

    let mut decondenser = config.into_decondenser();

    if let Some(indent) = indent {
        decondenser = match indent.parse::<usize>() {
            Ok(n_spaces) => decondenser.indent(n_spaces),
            Err(_) => decondenser.indent(indent),
        };
    }

    if let Some(max_line_size) = max_line_size {
        decondenser = decondenser.max_line_size(max_line_size);
    }

    if let Some(no_break_size) = no_break_size {
        decondenser = decondenser.no_break_size(no_break_size);
    }

    let input = read_input(&input)?;
    let output_str = decondenser.decondense(&input);

    write_output(&output, output_str)
}

fn config_or_default(config: Option<&Path>, files: &mut Files) -> Result<Config> {
    Ok(match config {
        Some(config) => Config::from_file(files, config)?.with_context(|| {
            format!(
                "Config file was not found at the specified path: '{}'",
                config.display()
            )
        })?,
        None => Config::discover(files)?.unwrap_or_default(),
    })
}

fn read_input(input: &str) -> Result<String> {
    if input != "-" {
        let content = std::fs::read_to_string(input)
            .with_context(|| format!("Failed to read file '{input}'"))?;

        return Ok(content);
    }

    let mut content = String::new();
    std::io::stdin()
        .read_to_string(&mut content)
        .context("Failed to read from stdin")?;

    Ok(content)
}

fn write_output(target: &str, content: String) -> Result {
    if target == "-" {
        println!("{content}");
        return Ok(());
    }

    std::fs::write(target, content)
        .with_context(|| format!("Failed to write to file '{target}'"))?;

    Ok(())
}
