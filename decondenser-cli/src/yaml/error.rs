use crate::Diagnostic;
use codespan_reporting::diagnostic::Label;
use marked_yaml::types::MarkedScalarNode;

pub(crate) struct Error {
    diagnostics: Vec<Diagnostic>,
}

impl From<Diagnostic> for Error {
    fn from(value: Diagnostic) -> Self {
        Self {
            diagnostics: vec![value],
        }
    }
}

impl From<marked_yaml::LoadError> for Error {
    fn from(err: marked_yaml::LoadError) -> Self {
        use marked_yaml::LoadError::*;

        let marker = match err {
            TopLevelMustBeMapping(marker)
            | TopLevelMustBeSequence(marker)
            | UnexpectedAnchor(marker)
            | MappingKeyMustBeScalar(marker)
            | UnexpectedTag(marker)
            | ScanError(marker, _) => marker,
            DuplicateKey(err) => {
                let span =
                    |node: &MarkedScalarNode| Some((*node.span().start()?, *node.span().end()?));

                let first_entry = span(&err.prev_key);
                let second_entry = span(&err.key);

                let Some((first, second)) = first_entry.zip(second_entry) else {
                    return Diagnostic::error()
                        .with_message(format!("found duplicate key {}", err.key.as_str()))
                        .into();
                };

                return Diagnostic::error()
                    .with_labels(vec![
                        Label::secondary(
                            first.0.source(),
                            first.0.character()..first.1.character(),
                        )
                        .with_message("first key"),
                        Label::primary(
                            second.0.source(),
                            second.0.character()..second.1.character(),
                        )
                        .with_message("duplicate key"),
                    ])
                    .into();
            }
        };

        Diagnostic::error()
            .with_label(
                Label::primary(marker.source(), marker.character()..marker.character() + 1)
                    .with_message(&format_args!("{err}")),
            )
            .into()
    }
}

impl Error {
    pub(crate) fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
