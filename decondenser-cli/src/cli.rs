use crate::config::Config;
use crate::{Files, Result};
use anyhow::Context;
use clap::{arg, value_parser};
use std::io::Read;
use std::path::{Path, PathBuf};

fn cli() -> clap::Command {
    command("decondenser", |cmd| {
        cmd.about("decondenser. More detailed docs: https://decondenser.dev")
            .styles(crate::styles::CLI_STYLES)
            .help_expected(true)
            .arg_required_else_help(true)
            .disable_help_subcommand(true)
            .disable_help_flag(true)
            .disable_version_flag(true)
            .propagate_version(true)
            .next_help_heading("Meta")
            .version(env!("CARGO_PKG_VERSION"))
            .long_version(None)
            .args([
                arg!(-h --help "Print help")
                    .action(clap::ArgAction::Help)
                    .global(true),
                arg!(-V --version "Print version")
                    .action(clap::ArgAction::Version)
                    .global(true),
            ])
            .subcommand_required(true)
            .subcommand(format_cli())
            .subcommand(unescape_cli())
    })
}

fn format_cli() -> clap::Command {
    command("fmt", |cmd| {
        cmd
        .about("Format text based on brackets nesting")
        .next_help_heading("Main")
        .args([
            arg!(--input <INPUT> "File path or - for stdin").default_value("-"),
            arg!(--output <OUTPUT> "File path or - for stdout").default_value("-"),
            arg!(
                --config <CONFIG>
                "Path to the config file [default: decondenser.yml in this or parent directories]"
            )
            .value_parser(value_parser!(PathBuf))
            .required(false),
        ])
        .next_help_heading("Formatting")
        .args([
            arg!(
                --indent <INDENT>
                "Number of spaces or string to use for indentation [default: 4]"
            ),
            arg!(
                --"max-line-size" <SIZE>
                "Best-effort max size of a line to fit into; see also --no-break-size \
                [default: 80]"
            ),
            arg!(
                --"no-break-size" <SIZE>
                "Lines shorter than this (ignoring indent) won't be broken \
                [default: --max-line-size / 2]"
            ),
        ])
    })
}

fn unescape_cli() -> clap::Command {
    command("unescape", |cmd| {
        cmd.about(
            "Unescape text based on common escapes syntax like JSON, Rust, Elixir, Python, etc.",
        )
        .arg(arg!(--input <INPUT> "File path or - for stdin").default_value("-"))
        .arg(arg!(--output <OUTPUT> "File path or - for stdout").default_value("-"))
    })
}

fn command(name: &'static str, configure: fn(clap::Command) -> clap::Command) -> clap::Command {
    let command = clap::Command::new(name).long_about(None);
    configure(command)
}

pub(crate) fn run(files: &mut Files) -> Result {
    let mut cli = cli().get_matches();

    let (subcommand, cli) = cli.remove_subcommand().unwrap();

    match subcommand.as_str() {
        "unescape" => unescape(cli),
        "fmt" => format(cli, files),
        _ => unreachable!("Unhandled subcommand: {subcommand}"),
    }
}

fn unescape(mut cli: clap::ArgMatches) -> Result {
    let input = cli.remove_one::<String>("input").unwrap();
    let output = cli.remove_one::<String>("output").unwrap();

    let input = read_input(&input)?;
    let output_str = decondenser::unescape(&input);

    write_output(&output, &output_str)
}

fn format(mut cli: clap::ArgMatches, files: &mut Files) -> Result {
    let input = cli.remove_one::<String>("input").unwrap();
    let output = cli.remove_one::<String>("output").unwrap();
    let config = cli.remove_one::<PathBuf>("config");
    let indent = cli.remove_one::<String>("indent");
    let max_line_size = cli.remove_one::<usize>("max-line-size");
    let no_break_size = cli.remove_one::<usize>("no-break-size");

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
    let output_str = decondenser.format(&input);

    write_output(&output, &output_str)
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

fn write_output(target: &str, content: &str) -> Result {
    if target == "-" {
        println!("{content}");
        return Ok(());
    }

    std::fs::write(target, content)
        .with_context(|| format!("Failed to write to file '{target}'"))?;

    Ok(())
}
