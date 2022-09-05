use proc_macro2::TokenStream;
use syn::{
    bracketed,
    parse::Parse,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, parsing::peek_keyword, Eq},
    Attribute, DeriveInput, Field, Lit, LitInt, LitStr, Path, Result as SynResult, Token,
};

use crate::model::{FieldAttribute, Model, ModelField, Port};

impl TryFrom<Attribute> for FieldAttribute {
    type Error = syn::Error;

    fn try_from(value: Attribute) -> Result<Self, Self::Error> {
        match value.path.get_ident() {
            Some(ident) => match ident.to_string().as_str() {
                "env_var" => parse_env_var(value),
                _ => Err(syn::Error::new_spanned(value, "Expected any of ...")), // TODO
            },
            None => Err(syn::Error::new_spanned(value, "Expected identifier")),
        }
    }
}

struct ContainerInput {
    properties: Vec<Property>,
}

impl Parse for ContainerInput {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        println!("tokens: {:#?}", input);

        let punctuated: Punctuated<Property, Token![,]> = Punctuated::parse_terminated(input)?;

        let properties: Vec<Property> = punctuated.into_iter().collect();

        Ok(ContainerInput { properties })
    }
}

enum Property {
    HealthCheckCommand(Path, Eq, LitStr),
    HealthCheckTimeout(Path, Eq, LitInt),
    Image(Path, Eq, LitStr),
    Ports(Path, Eq, token::Bracket, Punctuated<LitPort, Token![,]>),
}

impl Parse for Property {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        let cursor = input.cursor();

        if peek_keyword(cursor, "image") {
            Ok(Property::Image(
                input.parse()?,
                input.parse()?,
                input.parse()?,
            ))
        } else if peek_keyword(cursor, "health_check_command") {
            Ok(Property::HealthCheckCommand(
                input.parse()?,
                input.parse()?,
                input.parse()?,
            ))
        } else if peek_keyword(cursor, "health_check_timeout") {
            Ok(Property::HealthCheckTimeout(
                input.parse()?,
                input.parse()?,
                input.parse()?,
            ))
        } else if peek_keyword(cursor, "ports") {
            let content;
            Ok(Property::Ports(
                input.parse()?,
                input.parse()?,
                bracketed!(content in input),
                Punctuated::parse_terminated(&content)?,
            ))
        } else {
            Err(input.error("Expected any of..."))
        }
    }
}

struct LitPort {
    source: LitInt,
    _colon: Token![:],
    target: LitInt,
}

impl Parse for LitPort {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        Ok(LitPort {
            source: input.parse()?,
            _colon: input.parse()?,
            target: input.parse()?,
        })
    }
}

struct FieldProperty {
    _operator: Eq,
    value: Lit,
}

impl Parse for FieldProperty {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        Ok(Self {
            _operator: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub fn parse_container(tokens: TokenStream) -> SynResult<Model> {
    let item_ast: DeriveInput = syn::parse2(tokens).unwrap();

    println!("ITEM AST: {:#?}", item_ast);

    parse_derive_input(item_ast)
}

fn parse_env_var(value: Attribute) -> SynResult<FieldAttribute> {
    let field_property: FieldProperty = syn::parse2(value.tokens)?;

    match field_property.value {
        Lit::Str(str) => Ok(FieldAttribute::EnvVar(str.value().to_string())),
        _ => Err(syn::Error::new_spanned(
            field_property.value,
            "Expected string literal",
        )),
    }
}

fn parse_derive_input(ast: DeriveInput) -> SynResult<Model> {
    let struct_name = ast.ident.to_string();
    let attr = get_container_attribute(&ast)?;
    let container_input: ContainerInput = attr.parse_args()?;
    let image = get_image_name(&container_input).expect("Expected at least an image property");
    let health_check_command = get_health_check_command(&container_input);
    let ports = get_ports(&container_input);

    let fields = parse_fields(get_fields(ast))?;

    Ok(Model {
        struct_name,
        image,
        health_check_command,
        ports,
        fields,
    })
}

fn get_health_check_command(container_input: &ContainerInput) -> Option<String> {
    container_input
        .properties
        .iter()
        .find_map(|property| match property {
            Property::HealthCheckCommand(_, _, command) => Some(command.value()),
            _ => None,
        })
}

fn get_image_name(container_input: &ContainerInput) -> Option<String> {
    container_input
        .properties
        .iter()
        .find_map(|property| match property {
            Property::Image(_, _, name) => Some(name.value()),
            _ => None,
        })
}

fn get_ports(container_input: &ContainerInput) -> Vec<Port> {
    container_input
        .properties
        .iter()
        .find_map(|property| match property {
            Property::Ports(_, _, _, ports) => Some(
                ports
                    .iter()
                    .map(|port| {
                        Port {
                            source: port.source.base10_parse().unwrap(), // TODO replace unwrap
                            target: port.target.base10_parse().unwrap(), // TODO replace unwrap
                        }
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or(Vec::new())
}

fn parse_fields(input: Vec<Field>) -> SynResult<Vec<ModelField>> {
    input.into_iter().map(|field| parse_field(field)).collect()
}

fn parse_field(field: Field) -> SynResult<ModelField> {
    let attrs = field
        .attrs
        .into_iter()
        .map(FieldAttribute::try_from)
        .filter_map(Result::ok)
        .collect::<Vec<FieldAttribute>>();

    Ok(ModelField {
        name: field.ident.unwrap().to_string(),
        attributes: attrs,
    })
}

fn get_fields(input: DeriveInput) -> Vec<Field> {
    match input.data {
        syn::Data::Struct(data) => data.fields.into_iter().collect(),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
}

fn get_container_attribute<'a>(input: &'a DeriveInput) -> SynResult<&'a Attribute> {
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

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::{
        model::{FieldAttribute, Model, ModelField, Port},
        parse::parse_container,
    };

    #[test]
    fn test_parse_container() {
        let tokens_in = quote! {
            #[derive(Default, Container)]
            #[container(
                image = "docker.io/library/nginx",
                health_check_command = "curl http://localhost || exit 1",
                health_check_timeout = 30000,
                ports = [8080:8080, 8081:8080]
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
                struct_name: "SimpleImage".to_string(),
                image: "docker.io/library/nginx".to_string(),
                health_check_command: Some("curl http://localhost || exit 1".to_string()),
                ports: vec![
                    Port {
                        source: 8080,
                        target: 8080
                    },
                    Port {
                        source: 8081,
                        target: 8080
                    }
                ],
                fields: vec![ModelField {
                    name: "password".to_string(),
                    attributes: vec![FieldAttribute::EnvVar("PASSWORD".to_string())]
                }]
            }
        );
    }
}
