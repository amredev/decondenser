use crate::config::Config;
use crate::{Files, Result};
use anyhow::Context;
use clap::{arg, value_parser};
use std::io::Read;
use std::path::PathBuf;

fn cli() -> clap::Command {
    clap::Command::new("decondenser")
        .about(
            "decondenser pretty-prints any text based on brackets nesting\n\
            docs: https://decondenser.dev",
        )
        .long_about(None)
        .version(env!("CARGO_PKG_VERSION"))
        .styles(crate::styles::CLI_STYLES)
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
        .arg(
            arg!(
                --lang <LANG>
                "language profile; either a built-in or custom language defined in the config"
            )
            .default_value("generic"),
        )
}

pub(crate) fn run(files: &mut Files) -> Result {
    let matches = cli().get_matches();

    let input = matches.get_one::<String>("input").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let config = matches.get_one::<PathBuf>("config");
    let lang = matches.get_one::<String>("lang").unwrap();

    let config = match config {
        Some(config) => Config::from_file(files, config)?.with_context(|| {
            format!(
                "Config file was not found at the specified path: '{}'",
                config.display()
            )
        })?,
        None => Config::discover(files)?.unwrap_or_default(),
    };
    let decondenser = config.into_decondenser(lang)?;

    let output_str = decondenser.decondense(&read_input(input)?);

    write_output(output, output_str)
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
        .with_context(|| "Failed to read from stdin")?;

    Ok(content)
}

fn write_output(output: &str, content: String) -> Result {
    if output == "-" {
        println!("{content}");
        return Ok(());
    }

    std::fs::write(output, content)
        .with_context(|| format!("Failed to write to file '{output}'"))?;

    Ok(())
}
