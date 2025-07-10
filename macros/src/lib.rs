use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{FnArg, Ident, ImplItem, ItemImpl, Pat, PatType, Token, parse_macro_input, parse_quote};

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

            attr.map(|attr| (attr, item_fn))
        } else {
            None
        }
    });

    #[expect(unused_variables)]
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

    let wf_arg = run_fn.sig.inputs.iter().find(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident == "wf"
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

    #[expect(unused_variables)]
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
                let event = #ty::try_from(event).map_err(|_| StepError::WrongEventType)?;
            }),
            ty.clone(),
        )
    } else {
        (None, parse_quote!(Immediate<Self::Workflow>))
    };

    let wf = if let Some(wf_arg) = wf_arg {
        let ty = if let FnArg::Typed(PatType { ty, .. }) = wf_arg {
            ty
        } else {
            return TokenStream::from(Error::missing_field("wf").write_errors());
        };
        Some(ty.clone())
    } else {
        None
    };

    quote! {
    #input
    impl Step for #step_type {
        type Workflow = #wf;
        type Event = #ty;
        async fn run_raw(
            &self,
            wf: Self::Workflow,
            event: Option<<Self::Workflow as Workflow>::Event>,
        ) -> Result<Option<StepWithSettings<<Self::Workflow as Workflow>::Step>>, StepError> {
            #event_extraction

            self.run(wf, #run_args).await
        }
    }
    }
    .into()
}

/// Expands to code that spawns the selected handlers for the given workflow type.
/// Usage: my_macro!(Workflow1, workspace_instance, active_step, next_step, handle_event_new);
#[proc_macro]
pub fn startup_workflow(input: TokenStream) -> TokenStream {
    struct Idents(Punctuated<Ident, Token![,]>);

    impl syn::parse::Parse for Idents {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Idents(Punctuated::parse_terminated(input)?))
        }
    }

    let Idents(input) = parse_macro_input!(input as Idents);

    let mut idents = input.into_iter();
    let workflow_ty = match idents.next() {
        Some(ty) => ty,
        None => {
            return Error::custom("Expected workflow type as first argument")
                .write_errors()
                .into();
        }
    };

    let handlers = idents.map(|ident| {
        let fn_name = syn::Ident::new(&format!("{}", ident), ident.span());
        quote! { #fn_name::<#workflow_ty>() }
    });

    let expanded = quote! {
        ( 
            #workflow_ty::control_router(sqlx_tx_state, &mut session).await?, 
            async {
                try_join!(
                    #(#handlers),*
                )
            }
        )
    };

    expanded.into()
}

// let router_1 = Workflow1::control_router(sqlx_tx_state, &mut session).await?;

//     let handlers_1 = async {
//         try_join!(
//             workspace_instance_worker::<Workflow1>(),
//             active_step_worker(Workflow1 {}),
//             next_step_worker::<Workflow1>(),
//             handle_event_new::<Workflow1>()
//         )
//     };
