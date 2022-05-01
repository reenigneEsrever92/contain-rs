extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ExprCall};

#[proc_macro_attribute]
pub fn container(args: TokenStream, item: TokenStream) -> TokenStream {
    println!("Image Attribute: \n\n");

    println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", item);

    let args_ast: ExprCall = syn::parse(args).unwrap();
    let item_ast: DeriveInput = syn::parse(item).unwrap();

    println!("ARGS AST: {:#?}", args_ast);
    println!("ITEM AST: {:#?}", item_ast);

    let result = generate_into_container(&item_ast, &args_ast);

    println!("OUTPUT: {}", result);

    result
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
