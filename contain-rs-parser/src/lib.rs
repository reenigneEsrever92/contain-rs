use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    parse2,
    spanned::Spanned,
    Attribute, DeriveInput, ExprAssign, ExprCall, ExprLit, LitStr, PathSegment, Result, Token,
};

pub fn container(tokens: TokenStream2) -> TokenStream2 {
    // println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", tokens);

    let model = parse_container(tokens);

    // let args_ast: ExprCall = syn::parse(args).unwrap();

    // let result = generate_into_container(&item_ast, &args_ast);
    println!("OUTPUT: {:#?}", model);

    // result
    TokenStream2::new()
}

fn parse_container(tokens: TokenStream2) -> Result<Model> {
    let item_ast: DeriveInput = syn::parse2(tokens).unwrap();

    println!("ITEM AST: {:#?}", item_ast);

    parse_derive_input(item_ast)
}

struct ContainerInput {
    properties: Vec<ContainerProperty>,
}

impl Parse for ContainerInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut properties: Vec<ContainerProperty> = Vec::new();

        // TODO parse container properties
        properties.push(input.parse()?);

        Ok(ContainerInput { properties })
    }
}

struct ContainerProperty {
    property_type: PropertyType,
    operator: Token![=],
    value: LitStr,
}

impl Parse for ContainerProperty {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        input.parse()
    }
}

enum PropertyType {
    Image,
    HealthCheckCommand,
}

#[derive(Debug, PartialEq, Eq)]
struct Model {
    image: String,
    env_vars: Vec<(String, String)>,
}

fn parse_derive_input(ast: DeriveInput) -> Result<Model> {
    let attr = get_container_attribute(&ast)?;
    // TODO cloning sucks
    let container_input: ContainerInput = parse2(attr.tokens.clone())?;

    let image: Vec<String> = container_input
        .properties
        .iter()
        .filter(|prop| match prop.property_type {
            PropertyType::Image => true,
            _ => false,
        })
        .map(|prop| prop.value.value())
        .collect();

    Ok(Model {
        image: image.first().unwrap().to_string(),
        env_vars: Vec::new(),
    })
}

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

    use crate::{parse_container, Model};

    #[test]
    fn test_parse_container() {
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

        let model = parse_container(tokens_in);

        assert_eq!(
            model.unwrap(),
            Model {
                image: "docker.io/library/nginx".to_string(),
                env_vars: vec![]
            }
        );
    }
}
