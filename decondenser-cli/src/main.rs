#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod cli;
mod config;
mod error;
mod styles;
mod yaml;

use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor};
use error::{Error, Result};
use std::process::ExitCode;

type Files = SimpleFiles<String, String>;

type Label = codespan_reporting::diagnostic::Label<usize>;
type Diagnostic = codespan_reporting::diagnostic::Diagnostic<usize>;

fn main() -> ExitCode {
    let mut files = Files::new();

    let Err(err) = cli::run(&mut files) else {
        return ExitCode::SUCCESS;
    };

    match err {
        Error::Diagnostic(diags) => {
            for diag in &diags {
                let writer = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
                let config = term::Config {
                    before_label_lines: 5,
                    after_label_lines: 1,
                    ..Default::default()
                };

                let result = term::emit(&mut writer.lock(), &config, &files, diag);
                if let Err(err) = result {
                    eprintln!("Failed to emit diagnostic: {err:#?}");
                }
            }
        }
        Error::Other(err) => {
            eprintln!("Error: {err:?}");
        }
    }

    ExitCode::FAILURE
}
