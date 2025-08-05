use bindings::exports::decondenser as wit;

/// The bindings are generated via `cargo component`:
/// <https://github.com/bytecodealliance/cargo-component>
#[allow(warnings)]
#[rustfmt::skip]
#[doc(hidden)]
mod bindings;

struct Component;

bindings::export!(Component with_types_in bindings);

impl wit::Guest for Component {
    fn unescape(_input: wit::UnescapeParams) -> wit::UnescapeOutput {
        todo!()
        // decondenser::Decondenser::generic().unescape(&input)
    }

    fn decondense(params: wit::DecondenseParams) -> wit::DecondenseOutput {
        let output = decondenser::Decondenser::generic().format(&params.input);

        wit::DecondenseOutput { output }
    }
}
