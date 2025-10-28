use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{ItemFn, LitStr, MetaNameValue, Path, parse_macro_input};

// Centralized error message constants to keep stderr expectations stable.
const UNSUPPORTED_SHOULD_PANIC: &str = "unsupported form in #[should_panic]; allowed: #[should_panic], #[should_panic(expected = \"...\"), #[should_panic = \"...\"], #[should_panic(\"...\")]";
const DUP_EXPECTED: &str = "duplicate #[should_panic] expected message";
const DUP_SHOULD_PANIC: &str = "duplicate #[should_panic] attribute";
const DUP_IGNORE: &str = "duplicate #[ignore] attribute";
const DUP_ATTR: &str = "duplicate attribute";
const IGNORE_UNSUPPORTED: &str = "only #[ignore] and #[ignore = \"...\"] forms are supported";
const EXPECTED_STRING_AFTER_EQUALS: &str = "expected string literal after =";

fn split_comma_separated_tokens(tokens: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    use proc_macro2::{TokenStream as Ts, TokenTree};

    let mut segments = Vec::new();
    let mut current = Ts::new();

    for token in tokens {
        match &token {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if !current.is_empty() {
                    segments.push(current);
                    current = Ts::new();
                }
            }
            _ => current.extend([token]),
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

fn parse_case_attributes(attr_tokens: TokenStream) -> Result<Vec<LitStr>, syn::Error> {
    if attr_tokens.is_empty() {
        return Ok(Vec::new());
    }

    let segments = split_comma_separated_tokens(attr_tokens.into());
    let mut parsed = Vec::with_capacity(segments.len());

    for segment in segments {
        if segment.is_empty() {
            continue;
        }

        if let Ok(lit) = syn::parse2::<LitStr>(segment.clone()) {
            parsed.push(lit);
            continue;
        }

        if let Ok(path) = syn::parse2::<Path>(segment.clone()) {
            let value = path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            parsed.push(LitStr::new(&value, path.span()));
            continue;
        }

        return Err(syn::Error::new_spanned(
            segment,
            "only string literals or bare identifiers are supported in #[test(...)]",
        ));
    }

    Ok(parsed)
}

// Helper that returns at most one attribute by name, or an error if duplicates are present.
fn single_attr<'a>(
    attrs: &'a [syn::Attribute],
    name: &str,
) -> Result<Option<&'a syn::Attribute>, syn::Error> {
    let matches: Vec<_> = attrs.iter().filter(|a| a.path().is_ident(name)).collect();
    if matches.len() > 1 {
        let msg = match name {
            "should_panic" => DUP_SHOULD_PANIC,
            "ignore" => DUP_IGNORE,
            _ => DUP_ATTR,
        };
        return Err(syn::Error::new_spanned(matches[1], msg));
    }
    Ok(matches.get(0).copied())
}

// Parsed state holder for #[should_panic(...)] list forms
struct ShouldPanicParseResult {
    expected: Option<String>,
    positional: Option<String>,
    positional_count: usize,
    errors: Vec<syn::Error>,
}

impl ShouldPanicParseResult {
    fn from_attr_list(attr: &syn::Attribute, list: &syn::MetaList) -> Self {
        let mut res = ShouldPanicParseResult {
            expected: None,
            positional: None,
            positional_count: 0,
            errors: Vec::new(),
        };

        let segments = split_comma_separated_tokens(list.tokens.clone());
        for segment in segments {
            // Try expected = "..." first to recover exact span for duplicate on the `expected` ident
            if let Ok(meta) = syn::parse2::<syn::Meta>(segment.clone()) {
                if let syn::Meta::NameValue(MetaNameValue {
                    path,
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }),
                    ..
                }) = meta
                {
                    if path.is_ident("expected") {
                        if res.expected.is_some() {
                            // Duplicate expected -> error on the `expected` identifier span
                            res.errors.push(syn::Error::new(path.span(), DUP_EXPECTED));
                        } else {
                            res.expected = Some(s.value());
                        }
                        continue;
                    }
                }
            }

            // Try positional string literal
            if let Ok(lit) = syn::parse2::<syn::LitStr>(segment.clone()) {
                res.positional_count += 1;
                if res.positional.is_none() {
                    res.positional = Some(lit.value());
                }
                continue;
            }

            // Unknown / unsupported segment
            res.errors
                .push(syn::Error::new_spanned(segment, UNSUPPORTED_SHOULD_PANIC));
        }

        // Mixed forms: both positional and expected present
        if res.errors.is_empty() && res.expected.is_some() && res.positional.is_some() {
            res.errors
                .push(syn::Error::new_spanned(attr, UNSUPPORTED_SHOULD_PANIC));
        }

        // More than one positional literal -> unsupported
        if res.errors.is_empty() && res.positional_count > 1 {
            res.errors
                .push(syn::Error::new_spanned(attr, UNSUPPORTED_SHOULD_PANIC));
        }

        res
    }
}

/// Handles four forms:
/// 1. `#[should_panic]`
/// 2. `#[should_panic(expected = "...")]`
/// 3. `#[should_panic = "..."]`
/// 4. `#[should_panic("...")]` (positional string literal)
fn parse_should_panic_attr(
    attrs: &[syn::Attribute],
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let attr = match single_attr(attrs, "should_panic") {
        Ok(Some(attr)) => attr,
        Ok(None) => return Ok(quote! { ::core::option::Option::None }),
        Err(e) => return Err(e),
    };

    let tokens = match &attr.meta {
        syn::Meta::Path(_) => {
            // Bare #[should_panic]
            quote! { ::core::option::Option::Some(::satchel::ShouldPanic { expected: ::core::option::Option::None }) }
        }
        syn::Meta::NameValue(MetaNameValue {
            value:
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }),
            ..
        }) => {
            // #[should_panic = "message"]
            let s = lit_str.value();
            quote! { ::core::option::Option::Some(::satchel::ShouldPanic { expected: ::core::option::Option::Some(#s) }) }
        }
        syn::Meta::NameValue(_) => {
            return Err(syn::Error::new_spanned(attr, EXPECTED_STRING_AFTER_EQUALS));
        }
        syn::Meta::List(list) => {
            // Use the structured parser to collect state and errors
            let parsed = ShouldPanicParseResult::from_attr_list(attr, list);
            if !parsed.errors.is_empty() {
                let combined = parsed
                    .errors
                    .into_iter()
                    .reduce(|mut acc, err| {
                        acc.combine(err);
                        acc
                    })
                    .unwrap();
                return Err(combined);
            }

            // expected value preference: named expected over positional
            let expected_value = if let Some(s) = parsed.expected {
                quote! { ::core::option::Option::Some(#s) }
            } else if let Some(s) = parsed.positional {
                quote! { ::core::option::Option::Some(#s) }
            } else {
                quote! { ::core::option::Option::None }
            };

            quote! { ::core::option::Option::Some(::satchel::ShouldPanic { expected: #expected_value }) }
        }
    };
    Ok(tokens)
}

/// Handles two forms:
/// 1. `#[ignore]` - simple ignore without reason
/// 2. `#[ignore = "..."]` - ignore with a reason string
fn parse_ignore_attr(attrs: &[syn::Attribute]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let attr = match single_attr(attrs, "ignore") {
        Ok(Some(attr)) => attr,
        Ok(None) => return Ok(quote! { ::core::option::Option::None }),
        Err(e) => return Err(e),
    };

    // Handle #[ignore = "reason"]
    if let syn::Meta::NameValue(MetaNameValue {
        value:
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }),
        ..
    }) = &attr.meta
    {
        let reason = lit_str.value();
        return Ok(quote! {
            ::core::option::Option::Some(::satchel::Ignore {
                reason: ::core::option::Option::Some(#reason),
            })
        });
    }

    // Handle #[ignore] without reason
    if let syn::Meta::Path(_) = &attr.meta {
        return Ok(quote! {
            ::core::option::Option::Some(::satchel::Ignore {
                reason: ::core::option::Option::None,
            })
        });
    }

    // Unsupported form - emit compile error
    Err(syn::Error::new_spanned(attr, IGNORE_UNSUPPORTED))
}

#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_test_or_bench(
        attr,
        item,
        quote! {::satchel::TestKind::Unit },
        "__SATCHEL_TEST_",
    )
}

#[proc_macro_attribute]
pub fn bench(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_test_or_bench(
        attr,
        item,
        quote! { ::satchel::TestKind::Benchmark },
        "__SATCHEL_BENCH_",
    )
}

fn expand_test_or_bench(
    attr: TokenStream,
    input: TokenStream,
    kind: proc_macro2::TokenStream,
    prefix: &str,
) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);

    let should_panic = match parse_should_panic_attr(&input_fn.attrs) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };

    let ignore = match parse_ignore_attr(&input_fn.attrs) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let static_name = format_ident!("{}{}", prefix, fn_name_str.to_uppercase());
    let case_attribute_literals = match parse_case_attributes(attr) {
        Ok(list) => list,
        Err(e) => return e.into_compile_error().into(),
    };

    // Remove should_panic and ignore attributes from the function since we've processed them
    input_fn
        .attrs
        .retain(|attr| !attr.path().is_ident("should_panic") && !attr.path().is_ident("ignore"));

    let expanded = quote! {
        #[linkme::distributed_slice(::satchel::test_harness::TESTS)]
        static #static_name: ::satchel::TestCase = ::satchel::TestCase {
            name: #fn_name_str,
            module_path: ::core::module_path!(),
            kind: #kind,
            test_fn: #fn_name,
            should_panic: #should_panic,
            ignore: #ignore,
            case_attributes: &[ #( #case_attribute_literals ),* ] as &'static [&'static str],
        };

        #input_fn
    };

    TokenStream::from(expanded)
}
