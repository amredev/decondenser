#![doc = include_str!("../README.md")]

mod cli;
mod config;
mod styling;

use clap::Parser;
use std::process::ExitCode;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() -> ExitCode {
    let Err(err) = cli::Cli::parse().run() else {
        return ExitCode::SUCCESS;
    };

    eprintln!("Error: {err:?}");

    ExitCode::FAILURE
}
