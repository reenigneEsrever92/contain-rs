use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    spanned::Spanned, Attribute, DeriveInput, ExprAssign, ExprCall, ExprLit, PathSegment, Result,
};

pub fn parse_container(item: TokenStream2) -> TokenStream2 {
    println!("Image Attribute: \n\n");

    // println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", item);

    // let args_ast: ExprCall = syn::parse(args).unwrap();
    let item_ast: DeriveInput = syn::parse2(item).unwrap();

    // println!("ARGS AST: {:#?}", args_ast);
    println!("ITEM AST: {:#?}", item_ast);

    let model = parse_derive_input(item_ast);

    // let result = generate_into_container(&item_ast, &args_ast);
    println!("OUTPUT: {:#?}", model);

    // result
    TokenStream2::new()
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

fn parse_container_attribute(attr: &Attribute) {}

fn get_container_attribute<'a>(input: &'a DeriveInput) -> Result<&'a Attribute> {
    let attrs = input
        .attrs
        .iter()
        .filter(|attr| {
            attr.path
                .segments
                .last()
                .map(|segment| segment.ident.to_string() == "container")
                .unwrap_or(false)
        })
        .fold(None, |left, right| match left {
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

fn generate_into_container(item: &syn::DeriveInput, container_builder: &ExprCall) -> TokenStream2 {
    let ident = &item.ident;

    quote! {
        #item

        impl IntoContainer for #ident {
            fn into_container(self) -> Container {
                #container_builder.into_container()
            }
        }
    }
}

fn env(args: TokenStream2, item: TokenStream2) -> TokenStream2 {
    println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", &item);

    let args_ast: ExprLit = syn::parse2(args).unwrap();
    let item_ast: DeriveInput = syn::parse2(item).unwrap();

    println!("ARGS AST: {:#?}", args_ast);
    println!("ITEM AST: {:#?}", item_ast);

    let output = generate_env_impl(&args_ast, &item_ast);

    println!("OUTPUT: {}", output);

    TokenStream2::new()
}

fn generate_env_impl(args: &ExprLit, item: &DeriveInput) -> TokenStream2 {
    let ident = &item.ident;

    quote! {
        #item


    }
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::parse_container;

    #[test]
    fn test_parse_derive() {
        let tokens_in = quote! {
            #[derive(Default, Container)]
            #[container(
                image = "docker.io/library/nginx",
                health_check_command = "curl http://localhost || exit 1",
                health_check_timeout = 30000
            )]
            struct SimpleImage {
                password: String,
            }
        };

        let tokens_out = parse_container(tokens_in);
    }
}
