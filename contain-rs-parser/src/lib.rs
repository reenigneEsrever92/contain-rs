use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Eq, Attribute, DeriveInput,
    ExprCall, Lit, Path, Result, Token,
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
        println!("tokens: {:#?}", input);

        let punctuated: Punctuated<ContainerProperty, Token![,]> =
            Punctuated::parse_terminated(input)?;

        let properties: Vec<ContainerProperty> = punctuated.into_iter().collect();

        Ok(ContainerInput { properties })
    }
}

struct ContainerProperty {
    property_type: Path,
    _operator: Eq,
    value: Lit,
}

impl Parse for ContainerProperty {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        println!("property: {:?}", input);
        Ok(Self {
            property_type: input.parse()?,
            _operator: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Model {
    image: String,
    health_check_command: String,
    env_vars: Vec<(String, String)>,
}

fn parse_derive_input(ast: DeriveInput) -> Result<Model> {
    let attr = get_container_attribute(&ast)?;
    let container_input: ContainerInput = attr.parse_args()?;

    Ok(Model {
        image: string_value(find_property(&container_input, "image").unwrap())?,
        env_vars: Vec::new(),
        health_check_command: string_value(
            find_property(&container_input, "health_check_command").unwrap(),
        )?,
    })
}

fn find_property<'a>(
    container_input: &'a ContainerInput,
    name: &str,
) -> Option<&'a ContainerProperty> {
    container_input
        .properties
        .iter()
        .find(|prop| match prop.property_type.get_ident() {
            Some(ident) => {
                if ident.to_string().as_str() == name {
                    true
                } else {
                    false
                }
            }
            None => todo!(),
        })
}

fn string_value(property: &ContainerProperty) -> Result<String> {
    match &property.value {
        Lit::Str(str) => Ok(str.value().to_string()),
        _ => Err(syn::Error::new_spanned(
            property.value.clone(),
            "Expected a string for the image name",
        )),
    }
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

#[allow(dead_code)]
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
                #[env_var = "PASSWORD"]
                password: String,
            }
        };

        let model = parse_container(tokens_in);

        assert_eq!(
            model.unwrap(),
            Model {
                image: "docker.io/library/nginx".to_string(),
                health_check_command: "curl http://localhost || exit 1".to_string(),
                env_vars: vec![]
            }
        );
    }
}
