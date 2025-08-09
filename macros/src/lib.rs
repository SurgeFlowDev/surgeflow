use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    ImplItem, ImplItemType, ItemImpl, Token, Type, parse_macro_input, punctuated::Punctuated,
};

/// Attribute macro to expand a Workflow impl with step!(...) and event!(...) into
/// concrete enums `<Name>Step` and `<Name>Event`, and rewrites associated types
/// to those generated enums.
#[proc_macro_attribute]
pub fn workflow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_item: ItemImpl = parse_macro_input!(item as ItemImpl);

    // Determine the workflow type name (impl Workflow for <Name>)
    let self_ty_ident = match &*impl_item.self_ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|seg| seg.ident.clone())
            .unwrap_or_else(|| Ident::new("Workflow", Span::call_site())),
        _ => Ident::new("Workflow", Span::call_site()),
    };

    let step_enum_ident = format_ident!("{}Step", self_ty_ident);
    let event_enum_ident = format_ident!("{}Event", self_ty_ident);

    // Extract the Step and Event type macros from associated types
    let mut step_types: Vec<Type> = Vec::new();
    let mut event_types: Vec<Type> = Vec::new();

    for item in &impl_item.items {
        if let ImplItem::Type(ImplItemType { ident, ty, .. }) = item {
            if ident == "Step" {
                step_types = extract_macro_types(ty, "step");
            } else if ident == "Event" {
                event_types = extract_macro_types(ty, "event");
            }
        }
    }

    // Rewrite associated types Step and Event to generated enums
    for item in &mut impl_item.items {
        if let ImplItem::Type(ImplItemType { ident, ty, .. }) = item {
            if ident == "Step" {
                *ty = Type::Verbatim(quote! { #step_enum_ident });
            } else if ident == "Event" {
                *ty = Type::Verbatim(quote! { #event_enum_ident });
            }
        }
    }

    // Build enum variants A, B, C, ... for steps and events
    let step_variants = make_letter_variants(&step_types);
    let event_variants = make_letter_variants(&event_types);

    // Always include Immediate(Immediate) for events, with serde(skip)
    let immediate_variant = quote! {
        #[serde(skip)]
        Immediate(Immediate),
    };

    // Generate enums
    let step_enum: TokenStream2 = quote! {
        #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, From, TryInto)]
        pub enum #step_enum_ident {
            #(#step_variants)*
        }
    };

    let event_enum: TokenStream2 = quote! {
        #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
        pub enum #event_enum_ident {
            #(#event_variants)*
            #immediate_variant
        }
    };

    // Output the rewritten impl followed by the generated enums
    let output = quote! {
        #impl_item
        #step_enum
        #event_enum
    };

    TokenStream::from(output)
}

fn extract_macro_types(ty: &Type, expected_macro: &str) -> Vec<Type> {
    match ty {
        Type::Macro(m) => {
            // Ensure macro path ends with expected ident (e.g., step or event)
            let path_ident_ok = m
                .mac
                .path
                .segments
                .last()
                .map(|s| s.ident == expected_macro)
                .unwrap_or(false);
            if !path_ident_ok {
                return Vec::new();
            }
            // Parse inner tokens as a comma-separated list of types
            syn::parse2::<MaybeParens<TypeList>>(m.mac.tokens.clone())
                .map(|list| list.0.0.into_iter().collect())
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

fn make_letter_variants(types: &[Type]) -> Vec<TokenStream2> {
    types
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let name = make_letters(i);
            let ident = Ident::new(&name, Span::call_site());
            quote! { #ident(#ty), }
        })
        .collect()
}

fn make_letters(mut i: usize) -> String {
    // Convert 0-based index to Excel-like letters: 0->A, 1->B, ... 25->Z, 26->AA, ...
    let mut s = String::new();
    i += 1; // 1-based
    while i > 0 {
        let rem = (i - 1) % 26;
        s.push((b'A' + rem as u8) as char);
        i = (i - 1) / 26;
    }
    s.chars().rev().collect()
}

// Helpers to parse comma-separated type lists, optionally wrapped in parentheses
struct TypeList(Punctuated<Type, Token![,]>);

impl Parse for TypeList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TypeList(Punctuated::<Type, Token![,]>::parse_terminated(
            input,
        )?))
    }
}

struct MaybeParens<T>(T);

impl<T: Parse> Parse for MaybeParens<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Ok(MaybeParens(content.parse()?))
        } else {
            Ok(MaybeParens(input.parse()?))
        }
    }
}
