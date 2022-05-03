extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Attribute, DeriveInput, ExprAssign, ExprCall, ExprLit, Result};

#[proc_macro_derive(Container, attributes(container, env_var))]
pub fn container(item: TokenStream) -> TokenStream {
    println!("Image Attribute: \n\n");

    // println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", item);

    // let args_ast: ExprCall = syn::parse(args).unwrap();
    let item_ast: DeriveInput = syn::parse(item).unwrap();

    // println!("ARGS AST: {:#?}", args_ast);
    println!("ITEM AST: {:#?}", item_ast);

    let model = parse_derive_input(item_ast);

    // let result = generate_into_container(&item_ast, &args_ast);

    println!("OUTPUT: {:#?}", model);

    // result

    TokenStream::new()
}

#[derive(Debug)]
struct Model {
    image: String,
    env_vars: Vec<(String, String)>,
}

fn parse_derive_input(ast: DeriveInput) -> Result<Model> {
    let attr = get_container_attribute(&ast)?;

    Ok(Model {
        image: "test".to_string(),
        env_vars: Vec::new(),
    })
}

fn parse_attribute(attr: &Attribute) {
    
}

fn get_container_attribute<'a>(input: &'a DeriveInput) -> Result<&'a Attribute> {
    let attrs = input.attrs.iter().fold(None, |left, right| match left {
        Some(thing) => match thing {
            Ok(_) => Some(Err(syn::Error::new(
                right.span(),
                "Expected only one container annotation",
            ))),
            Err(mut e) => {
                e.combine(syn::Error::new(
                    right.span(),
                    "Expected only one container annotation",
                ));

                Some(Err(e))
            }
        },
        None => Some(Ok(right)),
    });

    match attrs {
        Some(result) => result,
        None => Err(syn::Error::new(
            input.span(),
            "Expected container annotation",
        )),
    }
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
