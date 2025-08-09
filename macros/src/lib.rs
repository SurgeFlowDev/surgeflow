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
    let step_letters: Vec<Ident> = (0..step_types.len())
        .map(|i| Ident::new(&make_letters(i), Span::call_site()))
        .collect();
    let step_variants: Vec<TokenStream2> = step_types
        .iter()
        .zip(step_letters.iter())
        .map(|(ty, letter)| quote! { #letter(#ty), })
        .collect();
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

    // Generate conversions between per-step error types and workflow step error type
    let mut per_step_error_items: Vec<TokenStream2> = Vec::new();
    for (i, ty) in step_types.iter().enumerate() {
        let variant_ident = &step_letters[i];

        // impl From<<StepX as Step>::Error> for <<Workflow as Workflow>::Step as WorkflowStep>::Error
        let from_impl = quote! {
            impl From<<#ty as Step>::Error> for <<#self_ty_ident as Workflow>::Step as WorkflowStep>::Error {
                fn from(error: <#ty as Step>::Error) -> Self {
                    <<#self_ty_ident as Workflow>::Step as WorkflowStep>::Error::#variant_ident(error)
                }
            }
        };

        // impl TryFrom<<<Workflow as Workflow>::Step as WorkflowStep>::Error> for <StepX as Step>::Error
        let tryfrom_impl = quote! {
            impl TryFrom<<<#self_ty_ident as Workflow>::Step as WorkflowStep>::Error> for <#ty as Step>::Error {
                type Error = ConvertingWorkflowStepToStepError;

                fn try_from(error: <<#self_ty_ident as Workflow>::Step as WorkflowStep>::Error) -> Result<Self, Self::Error> {
                    type WfError = <<#self_ty_ident as Workflow>::Step as WorkflowStep>::Error;
                    match error {
                        WfError::#variant_ident(e) => Ok(e),
                        _ => Err(ConvertingWorkflowStepToStepError),
                    }
                }
            }
        };

        per_step_error_items.push(from_impl);
        per_step_error_items.push(tryfrom_impl);
    }

    // Output the rewritten impl followed by the generated enums and conversions
    let output = quote! {
        #impl_item
        #step_enum
        #event_enum
        #(#per_step_error_items)*
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

// (no longer needed)

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
