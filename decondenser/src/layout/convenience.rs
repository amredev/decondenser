//! This code was originally adapted from the other codebase. See the parent
//! module's doc comment for more references.

use super::token::{Begin, Break, BreaksKind};
use super::{BreakParams, Layout, SIZE_INFINITY};

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

    pub(crate) fn space(&mut self) {
        self.spaces(1);
    }
}
