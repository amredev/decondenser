use super::{Deserialize, Errors, Result};
use crate::Diagnostic;
use crate::yaml::error::primary_label;
use hashlink::LinkedHashMap;
use marked_yaml::Node;
use marked_yaml::types::{MarkedMappingNode, MarkedScalarNode};
use std::collections::BTreeSet;
use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

type Dict = LinkedHashMap<MarkedScalarNode, Node>;

pub(crate) trait NodeExt: Sized {
    fn any_of<T>(self) -> AnyOfCtx<T>;
    fn object<T>(self, f: impl FnOnce(&mut Object) -> T) -> Result<T> {
        self.any_of().object(f).finish()
    }
    fn string(self) -> Result<String> {
        self.any_of().string(Ok).finish()
    }
    fn scalar<N: FromStr<Err: Display>>(self) -> Result<N> {
        self.any_of().scalar_from_str(Ok).finish()
    }
    fn enumeration<T>(self, variants: &[(&'static str, fn() -> T)]) -> Result<T> {
        self.any_of().enumeration(variants).finish()
    }
}

impl NodeExt for Node {
    fn any_of<T>(self) -> AnyOfCtx<T> {
        AnyOfCtx::new(self)
    }
}

pub(crate) struct AnyOfCtx<T>(AnyOfState<T>);

enum AnyOfState<T> {
    Done(Result<T>),
    Pending(PendingAnyOf),
}

struct PendingAnyOf {
    node: Node,
    allowed_types: TypesSet,
}

impl<T> AnyOfCtx<T> {
    fn new(node: Node) -> Self {
        Self(AnyOfState::Pending(PendingAnyOf {
            node,
            allowed_types: TypesSet::Empty,
        }))
    }

    pub(crate) fn object(self, f: impl FnOnce(&mut Object) -> T) -> Self {
        let Self(AnyOfState::Pending(mut pending)) = self else {
            return self;
        };

        let Node::Mapping(mapping) = pending.node else {
            pending.allowed_types.insert("an object");
            return Self(AnyOfState::Pending(pending));
        };

        let mut object = Object::new(mapping);
        let value = f(&mut object);
        Self(AnyOfState::Done(object.propagate_errors().map(|()| value)))
    }

    pub(crate) fn dict(self, f: impl FnOnce(Dict) -> Result<T>) -> Self {
        let Self(AnyOfState::Pending(mut pending)) = self else {
            return self;
        };

        let Node::Mapping(mut mapping) = pending.node else {
            pending
                .allowed_types
                .insert("an object with arbitrary keys");
            return Self(AnyOfState::Pending(pending));
        };

        Self(AnyOfState::Done(f(std::mem::take(&mut *mapping))))
    }

    pub(crate) fn array(self, f: impl FnOnce(Vec<Node>) -> Result<T>) -> Self {
        let Self(AnyOfState::Pending(mut pending)) = self else {
            return self;
        };

        let Node::Sequence(mut sequence) = pending.node else {
            pending.allowed_types.insert("an array");
            return Self(AnyOfState::Pending(pending));
        };

        Self(AnyOfState::Done(f(std::mem::take(&mut *sequence))))
    }

    pub(crate) fn string(self, f: impl FnOnce(String) -> Result<T>) -> Self {
        self.scalar_with_parser(
            "a string",
            |scalar| Ok::<_, Infallible>(scalar.as_str().to_owned()),
            f,
        )
    }

    pub(crate) fn usize(self, f: impl FnOnce(usize) -> Result<T>) -> Self {
        self.scalar_from_str(f)
    }

    fn scalar_from_str<S: FromStr<Err: Display>>(
        self,
        convert: impl FnOnce(S) -> Result<T>,
    ) -> Self {
        self.scalar_with_parser(
            std::any::type_name::<S>(),
            |scalar| scalar.parse::<S>(),
            convert,
        )
    }

    fn scalar_with_parser<U, E>(
        self,
        ty: &'static str,
        parse: impl FnOnce(&MarkedScalarNode) -> Result<U, E>,
        convert: impl FnOnce(U) -> Result<T>,
    ) -> Self {
        let Self(AnyOfState::Pending(mut pending)) = self else {
            return self;
        };

        let Node::Scalar(scalar) = &mut pending.node else {
            pending.allowed_types.insert(ty);
            return Self(AnyOfState::Pending(pending));
        };

        let Ok(value) = parse(scalar) else {
            pending.allowed_types.insert(ty);
            return Self(AnyOfState::Pending(pending));
        };

        Self(AnyOfState::Done(convert(value)))
    }

    pub(crate) fn enumeration(self, variants: &[(&'static str, fn() -> T)]) -> Self {
        let Self(AnyOfState::Pending(mut pending)) = self else {
            return self;
        };

        let into_pending = |mut pending: PendingAnyOf| {
            for (name, _) in variants {
                pending.allowed_types.insert(name);
            }
            Self(AnyOfState::Pending(pending))
        };

        let Node::Scalar(scalar) = &mut pending.node else {
            return into_pending(pending);
        };

        let string = scalar.as_str();

        let index = variants.iter().find(|(name, _)| *name == string);

        if let Some((_, variant)) = index {
            Self(AnyOfState::Done(Ok(variant())))
        } else {
            into_pending(pending)
        }
    }

    pub(crate) fn finish(self) -> Result<T> {
        let pending = match self.0 {
            AnyOfState::Done(result) => return result,
            AnyOfState::Pending(pending) => pending,
        };

        let allowed_types = pending.allowed_types.into_vec().join(" or ");

        Err(Errors::unexpected_type(&pending.node, &allowed_types))
    }
}

// Single-element optimization to avoid allocations in case only one type is allowed.
enum TypesSet {
    Empty,
    Single(&'static str),
    Multiple(BTreeSet<&'static str>),
}

impl TypesSet {
    fn insert(&mut self, key: &'static str) {
        match self {
            Self::Empty => *self = Self::Single(key),
            Self::Single(existing) => {
                if *existing != key {
                    *self = Self::Multiple(<_>::from_iter([existing, key]));
                }
            }
            Self::Multiple(set) => {
                set.insert(key);
            }
        }
    }

    fn into_vec(self) -> Vec<&'static str> {
        match self {
            Self::Empty => vec![],
            Self::Single(key) => vec![key],
            Self::Multiple(set) => set.into_iter().collect(),
        }
    }
}

pub(crate) struct Object {
    mapping: MarkedMappingNode,
    errors: Errors,
    allowed_keys: BTreeSet<&'static str>,
}

impl Object {
    fn new(mapping: MarkedMappingNode) -> Self {
        Self {
            mapping,
            errors: Errors::default(),
            allowed_keys: BTreeSet::new(),
        }
    }

    /// Returns the [`Default`] instance of the value if it's missing. This is
    /// to avoid failing fast and allow collecting more errors if possible. They
    /// will still be reported via [`Self::propagate_errors()`].
    pub(crate) fn required<T: Deserialize + Default>(&mut self, key: &'static str) -> T {
        self.allowed_keys.insert(key);

        if let Some(val) = self.mapping.remove(key) {
            return T::deserialize(val).unwrap_or_else(|errs| {
                self.errors.extend([errs]);
                Default::default()
            });
        }

        let diag = Diagnostic::error()
            .with_message(format!("missing key '{key}'"))
            .with_labels(vec![
                primary_label(*self.mapping.span()).with_message("object with missing key"),
            ]);

        self.errors.push(diag);

        Default::default()
    }

    pub(crate) fn optional<T: Deserialize>(&mut self, key: &'static str) -> Option<T> {
        self.allowed_keys.insert(key);

        let val = self.mapping.remove(key)?;

        T::deserialize(val).map(Some).unwrap_or_else(|err| {
            self.errors.extend([err]);
            None
        })
    }

    pub(crate) fn propagate_errors(mut self) -> Result {
        if !self.mapping.is_empty() {
            self.errors
                .push_unexpected_keys(self.mapping.keys(), self.allowed_keys);
        }

        self.errors.into_result()
    }
}
