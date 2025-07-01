/// A sealed struct to future-proof the trait method signatures and prevent
/// users from implementing the traits of this crate. See the guide:
/// <https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/>
#[expect(unnameable_types, missing_debug_implementations)]
pub struct Sealed;

#[cfg(feature = "unstable")]
impl super::Decondenser {
    /// Display the layout using special characters in the output:
    /// - `«»` - groups with [`BreakStyle::Consistent`]
    /// - `‹›` - groups with [`BreakStyle::Compact`]
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
