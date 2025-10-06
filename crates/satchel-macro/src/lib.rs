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
            let mut has_expected = false;

            // Handle three forms:
            // 1. #[should_panic(expected = "...")]
            // 2. #[should_panic("...")]
            // 3. #[should_panic = "..."]

            // First check for #[should_panic = "..."]
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        expected_msg = Some(lit_str.value());
                        has_expected = true;
                    }
                }
            } else {
                // Try to parse as #[should_panic(expected = "...")] or #[should_panic("...")]
                let _ = attr.parse_nested_meta(|meta| {
                    // Check for a direct string literal (form #2: #[should_panic("...")])
                    if let Ok(lit) = meta.value() {
                        if let Ok(s) = lit.parse::<syn::LitStr>() {
                            expected_msg = Some(s.value());
                            has_expected = true;
                        }
                    } else if meta.path.is_ident("expected") {
                        // Form #1: #[should_panic(expected = "...")]
                        has_expected = true;
                        if let Ok(lit) = meta.value() {
                            if let Ok(s) = lit.parse::<syn::LitStr>() {
                                expected_msg = Some(s.value());
                            }
                        }
                    }
                    Ok(())
                });
            }

            let expected = if has_expected {
                match expected_msg {
                    Some(msg) => quote! { ::core::option::Option::Some(#msg) },
                    None => quote! { ::core::option::Option::None },
                }
            } else {
                quote! { ::core::option::Option::None }
            };
            quote! {
                ::core::option::Option::Some(::satchel::ShouldPanic {
                    expected: #expected,
                })
            }
        })
        .unwrap_or(quote! { ::core::option::Option::None });

    let expanded = quote! {
        #[linkme::distributed_slice(::satchel::test_harness::TESTS)]
        static #static_name: ::satchel::TestCase = ::satchel::TestCase {
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
