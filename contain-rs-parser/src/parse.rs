use std::time::Duration;

use proc_macro2::{Ident, TokenStream};
use quote::__private::ext::RepToTokensExt;
use syn::{
    bracketed,
    parse::Parse,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, parsing::peek_keyword, Eq},
    Attribute, DeriveInput, Field, Lit, LitInt, LitStr, Path, Result as SynResult, Token,
};

use crate::model::{
    Command, FieldAttribute, FieldType, HealthCheck, Model, ModelField, WaitLog, WaitTime,
};

// impl TryFrom<Attribute> for FieldAttribute {
//     type Error = syn::Error;

//     fn try_from(value: Attribute) -> Result<Self, Self::Error> {
//         match value.path.get_ident() {
//             Some(ident) => match ident.to_string().as_str() {
//                 "env_var" => Ok(FieldAttribute::EnvVar(parse_field_attribute_value(value)?)),
//                 "arg" => Ok(FieldAttribute::Arg(parse_field_attribute_value(value)?)),
//                 "port" => parse_port_mapping(value),
//                 _ => Err(syn::Error::new_spanned(
//                     value,
//                     "Expected any of: arg or env_var",
//                 )),
//             },
//             None => Err(syn::Error::new_spanned(value, "Expected identifier")),
//         }
//     }
// }

struct ContainerInput {
    properties: Vec<Property>,
}

impl Parse for ContainerInput {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        let punctuated: Punctuated<Property, Token![,]> = Punctuated::parse_terminated(input)?;
        let properties: Vec<Property> = punctuated.into_iter().collect();

        Ok(ContainerInput { properties })
    }
}

enum Property {
    Command(Path, Eq, token::Bracket, Punctuated<LitStr, Token![,]>),
    HealthCheckCommand(Path, Eq, LitStr),
    HealthCheckTimeout(Path, Eq, LitInt),
    Image(Path, Eq, LitStr),
    Ports(Path, Eq, token::Bracket, Punctuated<LitPort, Token![,]>),
    WaitTime(Path, Eq, LitInt),
    WaitLog(Path, Eq, LitStr),
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
        } else if peek_keyword(cursor, "command") {
            let content;
            Ok(Property::Command(
                input.parse()?,
                input.parse()?,
                bracketed!(content in input),
                Punctuated::parse_terminated(&content)?,
            ))
        } else if peek_keyword(cursor, "wait_time") {
            Ok(Property::WaitTime(
                input.parse()?,
                input.parse()?,
                input.parse()?,
            ))
        } else if peek_keyword(cursor, "wait_log") {
            Ok(Property::WaitLog(
                input.parse()?,
                input.parse()?,
                input.parse()?,
            ))
        } else {
            Err(input.error("Expected any of: \"image\", \"command\", \"healt_check_command\", \"health_check_timeout\", \"ports\""))
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

#[derive(Debug)]
struct FieldProperty {
    ident: Ident,
    _operator: Eq,
    value: Lit,
}

impl Parse for FieldProperty {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        Ok(Self {
            ident: input.parse()?,
            _operator: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub fn parse_container(tokens: TokenStream) -> SynResult<Model> {
    let item_ast: DeriveInput = syn::parse2(tokens).unwrap();
    parse_derive_input(item_ast)
}

fn parse_field_attribute_value(value: Attribute) -> SynResult<String> {
    let field_property: FieldProperty = syn::parse2(value.tokens)?;

    match field_property.value {
        Lit::Str(str) => Ok(str.value()),
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
    let health_check = get_health_check_command(&container_input);
    // let ports = get_ports(&container_input);
    let command = get_command(&container_input);
    let wait_time = get_wait_time(&container_input)?;
    let wait_log = get_wait_log(&container_input);

    let fields = to_model_fields(parse_fields(get_fields(ast))?)?;

    Ok(Model {
        command,
        struct_name,
        image,
        health_check,
        fields,
        wait_time,
        wait_log,
    })
}

fn to_model_fields(fields: Vec<(Field, Vec<FieldProperty>)>) -> SynResult<Vec<ModelField>> {
    fields
        .into_iter()
        .map(|field| to_model_field(field))
        .collect::<SynResult<Vec<ModelField>>>()
}

fn to_model_field(field: (Field, Vec<FieldProperty>)) -> SynResult<ModelField> {
    let attributes = field
        .1
        .iter()
        .map(|property| match property.ident.to_string().as_str() {
            "env_var" => {
                if let Lit::Str(value) = &property.value {
                    Ok(FieldAttribute::EnvVar(value.value()))
                } else {
                    Err(syn::Error::new_spanned(
                        property.value.clone(),
                        "Expectet a String literal",
                    ))
                }
            }
            "arg" => {
                if let Lit::Str(value) = &property.value {
                    Ok(FieldAttribute::Arg(value.value()))
                } else {
                    Err(syn::Error::new_spanned(
                        property.value.clone(),
                        "Expected a String literal",
                    ))
                }
            }
            "port" => {
                if let Lit::Int(value) = &property.value {
                    Ok(FieldAttribute::Port(value.base10_parse()?))
                } else {
                    Err(syn::Error::new_spanned(
                        property.value.clone(),
                        "Expected an Integer literal",
                    ))
                }
            }
            _ => Err(syn::Error::new_spanned(
                property.value.clone(),
                "Expected any of: arg or env_var or port",
            )),
        })
        .collect::<SynResult<Vec<FieldAttribute>>>()?;

    let r#type: FieldType = parse_field_type(field.0.ty)?;

    Ok(ModelField {
        name: field.0.ident.unwrap().to_string(),
        r#type,
        attributes,
    })
}

fn get_wait_log(container_input: &ContainerInput) -> Option<WaitLog> {
    container_input
        .properties
        .iter()
        .find_map(|property| match property {
            Property::WaitLog(_, _, message) => Some(WaitLog {
                message: message.value(),
            }),
            _ => None,
        })
}

fn get_wait_time(container_input: &ContainerInput) -> SynResult<Option<WaitTime>> {
    container_input
        .properties
        .iter()
        .find_map(|property| match property {
            Property::WaitTime(_, _, duration) => {
                Some(duration.base10_parse().map(|duration| WaitTime {
                    time: Duration::from_millis(duration),
                }))
            }
            _ => None,
        })
        .map_or(Ok(None), |v| v.map(Some))
}

fn get_command(container_input: &ContainerInput) -> Option<Command> {
    container_input
        .properties
        .iter()
        .find_map(|property| match property {
            Property::Command(_, _, _, args) => Some(Command {
                args: args.iter().map(|lit| lit.value()).collect(),
            }),
            _ => None,
        })
}

fn get_health_check_command(container_input: &ContainerInput) -> Option<HealthCheck> {
    container_input
        .properties
        .iter()
        .find_map(|property| match property {
            Property::HealthCheckCommand(_, _, command) => {
                Some(HealthCheck::Command(command.value()))
            }
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

// fn get_ports(container_input: &ContainerInput) -> Vec<Port> {
//     container_input
//         .properties
//         .iter()
//         .find_map(|property| match property {
//             Property::Ports(_, _, _, ports) => Some(
//                 ports
//                     .iter()
//                     .map(|port| {
//                         Port {
//                             source: port.source.base10_parse().unwrap(), // TODO replace unwrap
//                             target: port.target.base10_parse().unwrap(), // TODO replace unwrap
//                         }
//                     })
//                     .collect(),
//             ),
//             _ => None,
//         })
//         .unwrap_or_default()
// }

fn parse_fields(input: Vec<Field>) -> SynResult<Vec<(Field, Vec<FieldProperty>)>> {
    input.into_iter().map(parse_field).collect()
}

fn parse_field(field: Field) -> SynResult<(Field, Vec<FieldProperty>)> {
    let attrs = field
        .attrs
        .iter()
        .map(|attr| attr.parse_args())
        .collect::<SynResult<Vec<FieldProperty>>>()?;

    Ok((field, attrs))
}

fn parse_field_type(ty: syn::Type) -> SynResult<FieldType> {
    match ty {
        syn::Type::Path(path) => {
            let ident = (
                path.path.segments.first(),
                path.path.segments.first().and_then(|segment| {
                    segment.arguments.next().and_then(|args| match args {
                        syn::PathArguments::AngleBracketed(bracketed) => {
                            match bracketed.args.first() {
                                Some(syn::GenericArgument::Type(syn::Type::Path(path))) => {
                                    Some(&path.path)
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                }),
            );

            match ident {
                (Some(_), None) => Ok(FieldType::Simple),
                (Some(segment), Some(_)) => {
                    if segment.ident == "Option" {
                        Ok(FieldType::Option)
                    } else {
                        Err(syn::Error::new_spanned(
                            path.path,
                            "Expected: Option or plain type",
                        ))
                    }
                }
                _ => Err(syn::Error::new_spanned(
                    path.path,
                    "Expected: Option or plain type",
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            ty,
            "Expected: Option or plain type",
        )),
    }
}

fn get_fields(input: DeriveInput) -> Vec<Field> {
    match input.data {
        syn::Data::Struct(data) => data.fields.into_iter().collect(),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
}

fn get_container_attribute(input: &DeriveInput) -> SynResult<&Attribute> {
    let attrs = input
        .attrs
        .iter()
        .filter(|attr| {
            attr.path
                .segments
                .last()
                .map(|segment| segment.ident == "container")
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
    use std::time::Duration;

    use quote::quote;

    use crate::{
        model::{FieldAttribute, FieldType, HealthCheck, Model, ModelField, WaitLog, WaitTime},
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
                wait_time = 2000,
                wait_log = "test"
            )]
            struct SimpleImage {
                #[contain_rs(env_var = "PASSWORD")]
                password: String,
                #[contain_rs(arg = "--arg")]
                arg: Option<String>,
                #[contain_rs(port = 8080)]
                web_port: Option<u32>,
            }
        };

        let model = parse_container(tokens_in);

        assert_eq!(
            model.unwrap(),
            Model {
                command: None,
                struct_name: "SimpleImage".to_string(),
                image: "docker.io/library/nginx".to_string(),
                health_check: Some(HealthCheck::Command(
                    "curl http://localhost || exit 1".to_string()
                )),
                fields: vec![
                    ModelField {
                        name: "password".to_string(),
                        r#type: FieldType::Simple,
                        attributes: vec![FieldAttribute::EnvVar("PASSWORD".to_string())]
                    },
                    ModelField {
                        name: "arg".to_string(),
                        r#type: FieldType::Option,
                        attributes: vec![FieldAttribute::Arg("--arg".to_string())]
                    },
                    ModelField {
                        name: "web_port".to_string(),
                        r#type: FieldType::Option,
                        attributes: vec![FieldAttribute::Port(8080)]
                    }
                ],
                wait_time: Some(WaitTime {
                    time: Duration::from_secs(2)
                }),
                wait_log: Some(WaitLog {
                    message: String::from("test")
                })
            }
        );
    }
}
