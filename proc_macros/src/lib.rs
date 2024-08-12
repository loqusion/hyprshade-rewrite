use std::env;

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, ItemFn,
};

lazy_static! {
    static ref IS_HYPRLAND_RUNNING: bool = env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some();
}

/// Attribute macro declaring a test function that will be ignored if the test is not running in a
/// Hyprland instance.
#[proc_macro_attribute]
pub fn hyprland_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    // Parsing the attribute is done to validate meta item syntax (in this case, lack thereof)
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct HyprlandTestAttribute {
    attrs: Vec<Attribute>,
}

impl Parse for HyprlandTestAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
        })
    }
}
