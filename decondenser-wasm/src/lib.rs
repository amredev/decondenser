mod into_core;

use bindings::exports::decondenser as wit;
use decondenser::Decondenser;

/// The bindings are generated via `cargo component`:
/// <https://github.com/bytecodealliance/cargo-component>
#[allow(warnings)]
#[rustfmt::skip]
#[doc(hidden)]
mod bindings;

struct Component;

bindings::export!(Component with_types_in bindings);

impl wit::Guest for Component {
    type Decondenser = Decondenser;

    fn unescape(input: String) -> String {
        decondenser::unescape(&input).into_owned()
    }
}

impl wit::GuestDecondenser for Decondenser {
    fn new(params: wit::DecondenserParams) -> Self {
        params.into_decondenser()
    }

    fn format(&self, input: String) -> String {
        eprintln!("EPRINTLN INVOKED");
        println!("PRINTLN INVOKED");
        // Delegate to the inherent method of the same name. No, it's not an
        // unconditional recursion, the compiler would report it otherwise  .
        self.format(&input)
    }
}
