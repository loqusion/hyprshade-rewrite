use std::env;

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, ItemFn,
};

lazy_static! {
    static ref IS_HYPRLAND_RUNNING: bool = env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some();
}

/// Attribute macro declaring a test function that will be ignored if the test is not running in a
/// Hyprland instance.
///
/// In other words, a test not running in a Hyprland instance should be roughly equivalent to the
/// following:
///
/// ```no_run
/// #[test]
/// #[ignore = "requires hyprland"]
/// fn test() { /* ... */ }
/// ```
///
/// **WARNING:** Once tests have been run, the decision to ignore tests will not be re-evaluated
/// until recompilation is triggered, e.g. by changing a file. This can lead to tests starting to
/// fail if the environment changes between runs.
#[proc_macro_attribute]
pub fn hyprland_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    // Verify there are no attribute arguments
    let _ = parse_macro_input!(attr as HyprlandTestAttribute);

    let ignore_attr = if *IS_HYPRLAND_RUNNING {
        None
    } else {
        Some(quote! {
            #[ignore = "requires hyprland"]
        })
    };

    let expanded = quote! {
        #[test]
        #ignore_attr
        #item
    };

    TokenStream::from(expanded)
}

#[derive(Debug, Clone)]
struct HyprlandTestAttribute {}

impl Parse for HyprlandTestAttribute {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(Self {})
    }
}
