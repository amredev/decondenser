/// Sealed trait used to specify "string-like" values that can be converted into
/// a [`String`].
#[expect(
    unnameable_types,
    reason = "
        Intentionally sealed for future-proofing the API. We may consider using
        a small-string optimized type in the future since most of the string
        configs in this crate are very short.
    "
)]
pub trait IntoString {
    fn into_string(self) -> String;
}

impl IntoString for String {
    fn into_string(self) -> String {
        self
    }
}

impl IntoString for &str {
    fn into_string(self) -> String {
        self.to_owned()
    }
}
