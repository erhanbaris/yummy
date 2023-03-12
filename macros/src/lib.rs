extern crate darling;

use proc_macro::*;
use quote::quote;
use syn::{ItemFn, parse_macro_input, AttributeArgs};
use darling::FromMeta;

#[derive(FromMeta)]
 struct MacroArgs {
    #[allow(dead_code)]
    name: String,

    #[darling(default)]
    no_socket: bool,

    #[darling(default)]
    no_return: bool
    
}

#[proc_macro_attribute]
pub fn plugin_api(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);

    let args = match MacroArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => { return TokenStream::from(e.write_errors()); }
    };

    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("expected function")
    };

    let ItemFn { block, ..} = fn_item;

    let has_message_sent_capability = !args.no_socket && !args.no_return;

    let (clone_socket, send_result, finish_with_error) = match has_message_sent_capability {
        true => {
            (quote! {
                let __socket__ = model.socket.clone();
                let __request_id__ = model.request_id.clone();
            },
            quote! {
               if let Err(result) = response.as_ref() {
                   __socket__.send(::model::WebsocketMessage::fail(__request_id__, result.to_string()).0)
               }
           },
           quote! {
                __socket__.send(::model::WebsocketMessage::fail(__request_id__, error.to_string()).0);
                return Err(error.into());
          })
        },
        false => (quote! { }, quote! { }, quote! { return; }),
    };

    let pre_api_path= proc_macro2::Ident::new(&format!("pre_{}", args.name), proc_macro2::Span::call_site());
    let post_api_path= proc_macro2::Ident::new(&format!("post_{}", args.name), proc_macro2::Span::call_site());

    // If the return is '()' than we should not return Ok or Err.
    let (response_block, execution_result_block) = match args.no_return {
        true => (quote! (), quote!{ true }),
        false => (quote! { response }, quote! { response.is_ok() }),
    };
    
    let body_block = quote! {
        {
            #clone_socket

            /* Execute pre_xxx api calls. If the call failed send error message to client */
            let model = match self.executer.#pre_api_path(model) {
                std::result::Result::Ok(model) => model,
                std::result::Result::Err(error) => {
                    log::error!("Pre error message: {:?}", error);
                    #finish_with_error
                }
            };

            /* Move all api codes into lambda. We want to get execution result and pass into the post_xxx api call. */
            let mut execute_api = || -> Self::Result {
                #block
            };

            /* Execute original api codes and get result */
            let response = execute_api();

            /* Send error message if the result is err */
            #send_result

            /* Call post_xxx api calls. If the post_xxx failed DO NOT send any additional error message to client */
            if let std::result::Result::Err(error) = self.executer.#post_api_path(model, #execution_result_block) {
                // Print only error message to console
                log::error!("Pre error message: {:?}", error);
            }

            // Api call finished and response back Result information
            #response_block
        }
    };

    fn_item.block.stmts.clear();
    fn_item.block.stmts.insert(0,syn::parse(body_block .into()).unwrap());

    use quote::ToTokens;
    item.into_token_stream().into()
}
