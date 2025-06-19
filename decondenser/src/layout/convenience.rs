//! This code was originally adapted from the other codebase. See the parent
//! module's doc comment for more references.

use super::token::{Begin, Break, BreaksKind};
use super::{BreakParams, Layout, SIZE_INFINITY};
use std::borrow::Cow;

impl Layout<'_> {
    pub(crate) fn begin_inconsistent(&mut self, indent: isize) {
        self.begin(indent, BreaksKind::Inconsistent);
    }

    pub(crate) fn begin_consistent(&mut self, indent: isize) {
        self.begin(indent, BreaksKind::Consistent);
    }

    fn spaces(&mut self, n: usize) {
        self.break_(BreakParams {
            blank_space: n,
            ..<_>::default()
        });
    }

    pub(crate) fn zerobreak(&mut self) {
        self.spaces(0);
    }

    pub(crate) fn space(&mut self) {
        self.spaces(1);
    }

    pub(crate) fn space_with_offset(&mut self, offset: isize) {
        self.break_(BreakParams {
            blank_space: 1,
            offset,
            ..<_>::default()
        });
    }

    pub(crate) fn hardbreak(&mut self) {
        self.spaces(SIZE_INFINITY as usize);
    }

    pub(crate) fn space_if_nonempty(&mut self) {
        self.break_(BreakParams {
            blank_space: 1,
            if_nonempty: true,
            ..<_>::default()
        });
    }

    pub(crate) fn hardbreak_if_nonempty(&mut self) {
        self.break_(BreakParams {
            blank_space: SIZE_INFINITY as usize,
            if_nonempty: true,
            ..<_>::default()
        });
    }

    pub(crate) fn neverbreak(&mut self) {
        self.break_(BreakParams {
            never_break: true,
            ..<_>::default()
        });
    }
}
