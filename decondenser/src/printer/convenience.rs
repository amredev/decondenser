//! This code was originally adapted from the other codebase. See the parent
//! module's doc comment for more references.

use super::{BeginToken, BreakToken, BreaksKind, Printer, SIZE_INFINITY};
use std::borrow::Cow;

impl Printer<'_> {
    pub(crate) fn begin_inconsistent(&mut self, indent: isize) {
        self.scan_begin(BeginToken {
            offset: indent,
            breaks_kind: BreaksKind::Inconsistent,
        });
    }

    pub(crate) fn begin_consistent(&mut self, indent: isize) {
        self.scan_begin(BeginToken {
            offset: indent,
            breaks_kind: BreaksKind::Consistent,
        });
    }

    pub(crate) fn word<S: Into<Cow<'static, str>>>(&mut self, wrd: S) {
        let s = wrd.into();
        self.scan_string(s);
    }

    fn spaces(&mut self, n: usize) {
        self.scan_break(BreakToken {
            blank_space: n,
            ..BreakToken::default()
        });
    }

    pub(crate) fn zerobreak(&mut self) {
        self.spaces(0);
    }

    pub(crate) fn space(&mut self) {
        self.spaces(1);
    }

    pub(crate) fn nbsp(&mut self) {
        self.word(" ");
    }

    pub(crate) fn hardbreak(&mut self) {
        self.spaces(SIZE_INFINITY as usize);
    }

    pub(crate) fn space_if_nonempty(&mut self) {
        self.scan_break(BreakToken {
            blank_space: 1,
            if_nonempty: true,
            ..BreakToken::default()
        });
    }

    pub(crate) fn hardbreak_if_nonempty(&mut self) {
        self.scan_break(BreakToken {
            blank_space: SIZE_INFINITY as usize,
            if_nonempty: true,
            ..BreakToken::default()
        });
    }

    pub(crate) fn neverbreak(&mut self) {
        self.scan_break(BreakToken {
            never_break: true,
            ..BreakToken::default()
        });
    }
}
