/// A sealed struct to future-proof the trait method signatures and prevent
/// users from implementing the traits of this crate. See the guide:
/// <https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/>
#[expect(unnameable_types, missing_debug_implementations)]
#[non_exhaustive]
pub struct Sealed;
