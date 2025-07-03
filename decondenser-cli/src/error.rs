use crate::{Diagnostic, yaml};

pub(crate) type Result<T = (), E = Error> = std::result::Result<T, E>;

pub(crate) enum Error {
    Diagnostic(Vec<Diagnostic>),
    Other(anyhow::Error),
}

impl From<yaml::Errors> for Error {
    fn from(errors: yaml::Errors) -> Self {
        Self::Diagnostic(errors.into_diagnostics().unwrap_or_default())
    }
}

impl From<Vec<Diagnostic>> for Error {
    fn from(diagnostic: Vec<Diagnostic>) -> Self {
        Self::Diagnostic(diagnostic)
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::Other(error)
    }
}
