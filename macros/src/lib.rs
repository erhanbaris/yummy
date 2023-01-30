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
    socket: bool,

    #[darling(default)]
    no_return: bool
    
}

#[proc_macro_attribute]
pub fn api(args: TokenStream, input: TokenStream) -> TokenStream {
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

    let (prepare_socket, send_message) = match args.socket {
        true => {
            (quote! { let __socket__ = model.socket.clone(); },
             quote! {
                if let Err(result) = response.as_ref() {
                    __socket__.send(general::model::WebsocketMessage::fail(result.to_string()).0)
                }
            })
        },
        false => (quote! { }, quote! { }),
    };

    let block = quote! {
        {
            #prepare_socket
            let mut call = || -> Self::Result {
                #block
            };
    
            let response = call();

            #send_message
            response
        }
    };

    fn_item.block.stmts.clear();
    fn_item.block.stmts.insert(0,syn::parse(block.into()).unwrap());

    use quote::ToTokens;
    item.into_token_stream().into()
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

    let (clone_socket, send_result) = match args.socket && !args.no_return {
        true => {
            (quote! { let __socket__ = model.socket.clone(); },
             quote! {
                if let Err(result) = response.as_ref() {
                    __socket__.send(general::model::WebsocketMessage::fail(result.to_string()).0)
                }
            })
        },
        false => (quote! { }, quote! { }),
    };

    let pre_api_path= proc_macro2::Ident::new(&format!("pre_{}", args.name), proc_macro2::Span::call_site());
    let post_api_path= proc_macro2::Ident::new(&format!("post_{}", args.name), proc_macro2::Span::call_site());

    // If the return is '()' than we should not return Ok or Err.
    let (response_block, execution_result_block) = match args.no_return {
        true => (quote! (), quote!{ true }),
        false => (quote! { response? }, quote! { response.is_ok() }),
    };
    
    let body_block = quote! {
        {
            #clone_socket
            let response = match self.executer.#pre_api_path(model) {
                std::result::Result::Ok(model) => {
                    let mut execute_api = || -> Self::Result {
                        #block
                    };
            
                    let response = execute_api();
        
                    #send_result
        
                    if let Err(error) = self.executer.#post_api_path(model, #execution_result_block) {
                        log::error!("Manager Api call failed: {:?}", error);
                    }

                    std::result::Result::Ok(response)
                }
                std::result::Result::Err(error) => Err(error)
            };

            #response_block
        }
    };

    fn_item.block.stmts.clear();
    fn_item.block.stmts.insert(0,syn::parse(body_block .into()).unwrap());

    use quote::ToTokens;
    item.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn simple_api(_: TokenStream, input: TokenStream) -> TokenStream {

    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("expected function")
    };

    let ItemFn { block, ..} = fn_item;

    let block = quote! {
        {
            let mut call = || -> Self::Result {
                #block
            };
    
            call();
        }
    };

    fn_item.block.stmts.clear();
    fn_item.block.stmts.insert(0,syn::parse(block.into()).unwrap());

    use quote::ToTokens;
    item.into_token_stream().into()
}