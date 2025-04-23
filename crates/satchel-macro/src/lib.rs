use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let static_name = format_ident!("__SATCHEL_TEST_{}", fn_name_str.to_uppercase());

    let expanded = quote! {
        #[linkme::distributed_slice(crate::test_harness::TESTS)]
        static #static_name: crate::test_harness::TestCase = crate::test_harness::TestCase {
            name: #fn_name_str,
            test_fn: #fn_name,
        };

        #input
    };

    TokenStream::from(expanded)
}
