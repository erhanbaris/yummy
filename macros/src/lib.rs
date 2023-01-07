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
    socket: bool
    
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
            let mut call = move || -> Self::Result {
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
pub fn simple_api(_: TokenStream, input: TokenStream) -> TokenStream {

    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("expected function")
    };

    let ItemFn { block, ..} = fn_item;

    let block = quote! {
        {
            let mut call = move || -> Self::Result {
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