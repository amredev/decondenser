use crate::Diagnostic;

pub(crate) type Result<T = (), E = Error> = std::result::Result<T, E>;

pub(crate) enum Error {
    Diagnostic(Vec<Diagnostic>),
    Other(anyhow::Error),
}

impl From<Vec<Diagnostic>> for Error {
    fn from(diagnostic: Vec<Diagnostic>) -> Self {
        Error::Diagnostic(diagnostic)
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::Other(error)
    }
}
