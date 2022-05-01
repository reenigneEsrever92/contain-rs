extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ExprCall, ExprLit};

#[proc_macro_derive(Container, attributes(container, env_var))]
pub fn container(item: TokenStream) -> TokenStream {
    println!("Image Attribute: \n\n");

    // println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", item);

    // let args_ast: ExprCall = syn::parse(args).unwrap();
    let item_ast: DeriveInput = syn::parse(item).unwrap();

    // println!("ARGS AST: {:#?}", args_ast);
    println!("ITEM AST: {:#?}", item_ast);

    // let result = generate_into_container(&item_ast, &args_ast);

    // println!("OUTPUT: {}", result);

    // result

    TokenStream::new()
}

fn generate_into_container(item: &syn::DeriveInput, container_builder: &ExprCall) -> TokenStream {
    let ident = &item.ident;

    quote! {
        #item

        impl IntoContainer for #ident {
            fn into_container(self) -> Container {
                #container_builder.into_container()
            }
        }
    }
    .into()
}

fn env(args: TokenStream, item: TokenStream) -> TokenStream {
    println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", &item);

    let args_ast: ExprLit = syn::parse(args).unwrap();
    let item_ast: DeriveInput = syn::parse(item).unwrap();

    println!("ARGS AST: {:#?}", args_ast);
    println!("ITEM AST: {:#?}", item_ast);
    
    let output = generate_env_impl(&args_ast, &item_ast);

    println!("OUTPUT: {}", output);

    TokenStream::new()
}

fn generate_env_impl(args: &ExprLit, item: &DeriveInput) -> TokenStream {
    let ident = &item.ident;

    quote! {
        #item

        
    }
    .into()
}