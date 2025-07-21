/// Panic but only in debug mode
macro_rules! debug_panic {
    ($($tt:tt)*) => {
        if cfg!(debug_assertions) {
            panic!($($tt)*)
        }
    };
}

pub(crate) use debug_panic;

/// Returns a &'static str with the path to the current scope, which includes
/// the current function, closure, or module and all of its parents.
macro_rules! scope_path {
    () => {{
        // The trick is to take the `type_name` of a symbol defined in the current
        // scope, because it includes the full path to that symbol with all the
        // parent modules and functions.
        struct S;
        let local_symbol_path = ::core::any::type_name::<S>();

        // Strip the `::S` suffix.
        &local_symbol_path[..local_symbol_path.len() - 3]
    }};
}

pub(crate) use scope_path;
