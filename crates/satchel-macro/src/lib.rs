use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_test_or_bench(item, quote! {::satchel::TestKind::Unit }, "__SATCHEL_TEST_")
}

#[proc_macro_attribute]
pub fn bench(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_test_or_bench(
        item,
        quote! { ::satchel::TestKind::Benchmark },
        "__SATCHEL_BENCH_",
    )
}

fn expand_test_or_bench(
    input: TokenStream,
    kind: quote::__private::TokenStream,
    prefix: &str,
) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let static_name = format_ident!("{}{}", prefix, fn_name_str.to_uppercase());

    let expanded = quote! {
        #[linkme::distributed_slice(crate::test_harness::TESTS)]
        static #static_name: ::satchel::test_harness::TestCase = ::satchel::test_harness::TestCase {
            name: #fn_name_str,
            module_path: ::std::module_path!(),
            kind: #kind,
            test_fn: #fn_name,
        };

        #input_fn
    };

    TokenStream::from(expanded)
}
