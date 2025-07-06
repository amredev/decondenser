use super::BreakStyle;

#[derive(Debug, Default)]
pub(super) struct Normalization {
    pub(super) blank: Option<Blank>,

    /// Tokens that belong to the previously opened groups that are not part
    /// of this normalization context.
    pub(super) suffixes: Vec<Suffix>,

    /// Groups that were started but not ended yet.
    pub(super) pending_groups: Vec<PendingGroup>,
}

#[derive(Debug)]
pub(super) struct PendingGroup {
    pub(super) break_style: BreakStyle,
    pub(super) indent: isize,
}

#[derive(Debug)]
pub(super) enum Suffix {
    End,
    Indent(isize),
}

#[derive(Debug)]
pub(super) enum Blank {
    Space(Space),
    Newline(usize),
}

#[derive(Debug)]
pub(super) struct Space {
    pub(super) size: usize,
    pub(super) breakable: bool,
}

impl Normalization {
    pub(super) fn begin(&mut self, break_style: BreakStyle) {
        self.pending_groups.push(PendingGroup {
            break_style,
            indent: 0,
        });
    }

    pub(super) fn end(&mut self) {
        if self.pending_groups.pop().is_none() {
            self.suffixes.push(Suffix::End);
        }
    }

    pub(super) fn indent(&mut self, diff: isize) {
        if diff == 0 {
            return;
        }

        if let Some(group) = self.pending_groups.last_mut() {
            group.indent += diff;
            return;
        }

        let Some(Suffix::Indent(indent)) = self.suffixes.last_mut() else {
            self.suffixes.push(Suffix::Indent(diff));
            return;
        };

        *indent += diff;

        if *indent == 0 {
            self.suffixes.pop();
        }
    }

    pub(super) fn blank(&mut self, blank: Blank) {
        let Some(current) = &mut self.blank else {
            self.blank = Some(blank);
            return;
        };

        match (current, blank) {
            (Blank::Space(current_space), Blank::Space(space)) => {
                current_space.breakable = current_space.breakable || space.breakable;
                current_space.size = std::cmp::max(current_space.size, space.size);
            }
            (current_blank @ Blank::Space(_), Blank::Newline(newline)) => {
                // Ignore trailing space and overwrite it with a newline
                *current_blank = Blank::Newline(newline);
            }
            (Blank::Newline(_), Blank::Space(_)) => {
                // Ignore leading space after a newline
            }
            (Blank::Newline(current_newlines), Blank::Newline(newlines)) => {
                *current_newlines = std::cmp::max(*current_newlines, newlines);
            }
        }
    }
}
