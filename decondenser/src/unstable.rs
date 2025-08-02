#[cfg(feature = "unstable")]
impl super::Decondenser {
    /// Display the layout using special characters in the output:
    /// - `«»` - groups with [`crate::BreakStyle::consistent()`]
    /// - `‹›` - groups with [`crate::BreakStyle::compact()`]
    pub fn debug_layout(mut self, value: bool) -> Self {
        self.debug_layout = value;
        self
    }

    /// Show indentation levels in the output using subscript number characters
    pub fn debug_indent(mut self, value: bool) -> Self {
        self.debug_indent = value;
        self
    }
}
