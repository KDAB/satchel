use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

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

    let should_panic = input_fn
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("should_panic"))
        .map(|attr| {
            let mut expected_msg = None;
            // Try to parse nested meta, but don't fail if there are none
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("expected") {
                    if let Ok(lit) = meta.value() {
                        if let Ok(s) = lit.parse::<syn::LitStr>() {
                            expected_msg = Some(s.value());
                        }
                    }
                }
                Ok(())
            });
            match expected_msg {
                Some(msg) => quote! { ::core::option::Option::Some(#msg) },
                None => quote! { ::core::option::Option::Some("") },
            }
        })
        .unwrap_or(quote! { ::core::option::Option::None });

    let expanded = quote! {
        #[linkme::distributed_slice(::satchel::test_harness::TESTS)]
        static #static_name: ::satchel::test_harness::TestCase = ::satchel::test_harness::TestCase {
            name: #fn_name_str,
            module_path: ::core::module_path!(),
            kind: #kind,
            test_fn: #fn_name,
            should_panic: #should_panic,
        };

        #input_fn
    };

    TokenStream::from(expanded)
}
