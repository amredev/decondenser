use crate::{Diagnostic, Label};
use codespan_reporting::diagnostic::LabelStyle;
use marked_yaml::types::MarkedScalarNode;
use marked_yaml::{Marker, Node, Span};

pub(crate) type Result<T = (), E = Errors> = std::result::Result<T, E>;

#[derive(Default)]
pub(crate) struct Errors {
    diagnostics: Vec<Diagnostic>,
}

impl Extend<Self> for Errors {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        self.diagnostics
            .extend(iter.into_iter().flat_map(|errors| errors.diagnostics));
    }
}

impl Extend<Diagnostic> for Errors {
    fn extend<T: IntoIterator<Item = Diagnostic>>(&mut self, iter: T) {
        self.diagnostics.extend(iter);
    }
}

impl Errors {
    pub(crate) fn into_diagnostics(self) -> Option<Vec<Diagnostic>> {
        (!self.diagnostics.is_empty()).then_some(self.diagnostics)
    }

    pub(crate) fn into_result(self) -> Result {
        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }

    pub(crate) fn unexpected_type(node: &Node, expected: &str) -> Self {
        let display_type = match node {
            Node::Mapping(_) => "an object",
            Node::Sequence(_) => "an array",
            Node::Scalar(scalar) => None
                .or_else(|| scalar.parse::<bool>().map(|_| "a boolean").ok())
                .or_else(|| scalar.parse::<u64>().map(|_| "u64").ok())
                .or_else(|| scalar.parse::<i64>().map(|_| "a negative i64").ok())
                .or_else(|| scalar.parse::<f64>().map(|_| "an f64").ok())
                .unwrap_or("a string"),
        };

        Self::unexpected_type_detailed(*node.span(), expected, display_type)
    }

    pub(crate) fn unexpected_type_detailed(
        span: Span,
        expected: &str,
        actual: impl std::fmt::Display,
    ) -> Self {
        Diagnostic::error()
            .with_message(format_args!("expected {expected} but found {actual}"))
            .with_labels(vec![primary_label(span).with_message("unexpected type")])
            .into()
    }

    #[expect(single_use_lifetimes, reason = "false positive, can't be elided here")]
    pub(super) fn push_unexpected_keys<'a>(
        &mut self,
        unexpected_keys: impl IntoIterator<Item = &'a MarkedScalarNode>,
        allowed_keys: impl IntoIterator<Item = &'static str>,
    ) {
        let unexpected_keys_labels = unexpected_keys
            .into_iter()
            .map(|key: &MarkedScalarNode| {
                primary_label(scalar_span(key)).with_message("unexpected key")
            })
            .collect::<Vec<_>>();

        let total = unexpected_keys_labels.len();
        let allowed_keys = allowed_keys.into_iter().collect::<Vec<_>>().join(", ");
        let s = if total == 1 { "" } else { "s" };

        let diag = Diagnostic::error()
            .with_message(format!("found {total} unexpected key{s}"))
            .with_labels(unexpected_keys_labels)
            .with_note(format!("allowed keys: {allowed_keys}"));

        self.push(diag);
    }

    pub(super) fn push(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }
}

impl From<Diagnostic> for Errors {
    fn from(value: Diagnostic) -> Self {
        Self {
            diagnostics: vec![value],
        }
    }
}

impl From<marked_yaml::LoadError> for Errors {
    fn from(err: marked_yaml::LoadError) -> Self {
        use marked_yaml::LoadError as E;

        let marker = match err {
            E::TopLevelMustBeMapping(marker)
            | E::TopLevelMustBeSequence(marker)
            | E::UnexpectedAnchor(marker)
            | E::MappingKeyMustBeScalar(marker)
            | E::UnexpectedTag(marker)
            | E::ScanError(marker, _) => marker,
            E::DuplicateKey(err) => {
                return Diagnostic::error()
                    .with_message("duplicate keys detected")
                    .with_labels(vec![
                        secondary_label(scalar_span(&err.prev_key)).with_message("first key"),
                        primary_label(scalar_span(&err.key)).with_message("duplicate key"),
                    ])
                    .into();
            }
        };

        #[expect(
            clippy::range_plus_one,
            reason = "Label expects an exclusive Range<usize>"
        )]
        Diagnostic::error()
            .with_label(
                Label::primary(marker.source(), marker.character()..marker.character() + 1)
                    .with_message(format_args!("{err}")),
            )
            .into()
    }
}

pub(crate) fn primary_label(span: Span) -> Label {
    label_with_style(LabelStyle::Primary, span)
}

pub(crate) fn secondary_label(span: Span) -> Label {
    label_with_style(LabelStyle::Secondary, span)
}

fn label_with_style(style: LabelStyle, span: Span) -> Label {
    // Yes, this is a bit of a hack, but span info may only be missing in case
    // if the nodes were generated programmatically instead of from a string. We
    // don't do that, we only parse an immutable config, so we can ignore the
    // complexity of handling error messages for programmatically generated
    // nodes. Related issue: https://github.com/kinnison/marked-data/issues/19
    let (source, start) = span
        .start()
        .map(|start| (start.source(), start.character()))
        .unwrap_or((0, 0));

    let end = span
        .end()
        .map(Marker::character)
        .unwrap_or_else(|| start + 1);

    Label::new(style, source, start..end)
}

fn scalar_span(scalar: &MarkedScalarNode) -> Span {
    let mut span: Span = *scalar.span();

    // There seems to be a bug in `marked_yaml` where the end span
    // is `None` for a simple scalar
    if let (Some(start), None) = (span.start(), span.end()) {
        span.set_end(Some(Marker::new(
            start.source(),
            start.character() + scalar.as_str().len(),
            0,
            0,
        )));
    }

    span
}
