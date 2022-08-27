use proc_macro2::TokenStream;
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Eq, Attribute, DeriveInput,
    Field, Lit, Path, Result as SynResult, Token,
};

use crate::model::{FieldAttribute, Model, ModelField};

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
    properties: Vec<ContainerProperty>,
}

impl Parse for ContainerInput {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
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
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        println!("property: {:?}", input);
        Ok(Self {
            property_type: input.parse()?,
            _operator: input.parse()?,
            value: input.parse()?,
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
    let image = string_value(find_property(&container_input, "image").unwrap())?;
    let health_check_command = find_property(&container_input, "health_check_command")
        .map(string_value)
        .map_or(Ok(None), |it| it.map(Some))?;
    let fields = parse_fields(get_fields(ast))?;

    Ok(Model {
        struct_name,
        image,
        health_check_command,
        fields,
    })
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

fn string_value(property: &ContainerProperty) -> SynResult<String> {
    match &property.value {
        Lit::Str(str) => Ok(str.value().to_string()),
        _ => Err(syn::Error::new_spanned(
            property.value.clone(),
            "Expected a string for the image name",
        )),
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
