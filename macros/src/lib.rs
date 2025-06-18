use darling::ast::NestedMeta;
use darling::{Error, FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Attribute, FnArg, ImplItem, ItemFn, ItemImpl, Pat, PatType, parse_quote};

#[derive(FromMeta)]
// #[darling(attributes(my_crate), forward_attrs(allow, doc, cfg))]
struct StepOpts {
    // ident: syn::Ident,
    // attrs: Vec<syn::Attribute>,
}

#[proc_macro_attribute]
pub fn step(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };

    let mut input = syn::parse_macro_input!(input as ItemImpl);

    let step_type = &input.self_ty;

    let run_fn = input.items.iter_mut().find_map(|item| {
        if let ImplItem::Fn(item_fn) = item {
            let attr = item_fn
                .attrs
                .extract_if(.., |attr| attr.path().is_ident("run"))
                .next();
            if let Some(attr) = attr {
                Some((attr, item_fn))
            } else {
                None
            }
        } else {
            None
        }
    });

    let (attr, run_fn) = match run_fn {
        Some((attr, run_fn)) => (attr, run_fn),
        None => {
            return TokenStream::from(Error::missing_field("run").write_errors());
        }
    };
    let event_arg = run_fn.sig.inputs.iter().find(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident == "event"
            } else {
                false
            }
        } else {
            false
        }
    });
    // TODO
    // let event_type = event_arg.and_then(|arg| {
    //     if let FnArg::Typed(PatType { ty, .. }) = arg {
    //         Some(ty)
    //     } else {
    //         None
    //     }
    // });

    let args = match StepOpts::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let mut run_args = Punctuated::<syn::Ident, syn::Token![,]>::new();
    if event_arg.is_some() {
        run_args.push(parse_quote!(event));
    }

    let (event_extraction, ty) = if let Some(event_arg) = event_arg {
        let ty = if let FnArg::Typed(PatType { ty, .. }) = event_arg {
            ty
        } else {
            return TokenStream::from(Error::missing_field("event").write_errors());
        };

        (
            Some(quote! {
                let event = if let Some(event) = event {
                    event
                } else {
                    return Err(StepError::Unknown);
                };
                let event = #ty::try_from(event).map_err(|_| StepError::Unknown)?;
            }),
            ty.clone(),
        )
    } else {
        (None, parse_quote!(Immediate))
    };
    /////////////////////////////////////////////////
    // remove wait_for_event. Every step should have an Event associated type, which can be "Immediate". from the outside
    // I can check the type's TypeId and see if it is "Immediate" or not. this way I can avoid the need for a wait_for_event method.
    /////////////////////////////////////////////////
    quote! {
    #input
    impl Step for #step_type {
        type Event = #ty;
        async fn run_raw(
            &self,
            event: Option<WorkflowEvent>,
        ) -> Result<Option<StepWithSettings<WorkflowStep>>, StepError> {
            #event_extraction

            self.run(#run_args).await
        }
    }
    }
    .into()
}
