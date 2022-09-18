use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::model::{FieldAttribute, Model, ModelField, Port};

pub fn generate_container(model: Model) -> TokenStream {
    let struct_name = format_ident!("{}", model.struct_name);
    let image_name = model.image;
    let fields = model.fields;
    let ports = model.ports;

    quote! {
        impl IntoContainer for #struct_name {
            fn into_container(self) -> Container {
                let image = Image::from_name(#image_name);
                let container = Container::from_image(image);
                #( #fields )*
                #( #ports )*
                container
            }
        }
    }
}

impl ToTokens for ModelField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attributes = &self.attributes;
        let field_tokens = generate_field_tokens(self, attributes);
        tokens.extend(quote! { #( #field_tokens )* })
    }
}

impl ToTokens for Port {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let source = self.source;
        let target = self.target;
        tokens.extend(quote! { container.map_port((#source, #target)); })
    }
}

fn generate_field_tokens(field: &ModelField, attributes: &[FieldAttribute]) -> Vec<TokenStream> {
    attributes
        .iter()
        .map(|attr| match attr {
            FieldAttribute::EnvVar(name) => generate_env_var(field, name),
        })
        .collect()
}

fn generate_env_var(field: &ModelField, name: &str) -> TokenStream {
    let field_name = format_ident!("{}", &field.name);

    match field.ty {
        crate::model::FieldType::Simple => quote! {
            container.env_var((#name, self.#field_name));
        },
        crate::model::FieldType::Option => quote! {
            self.#field_name.and_then(|value| container.env_var((#name, value)));
        },
    }
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::{generate::generate_container, parse::parse_container};

    #[test]
    fn test_generate() {
        let tokens_in = quote! {
            #[derive(Container)]
            #[container(
                image = "docker.io/library/nginx",
            )]
            struct Nginx;
        };

        let model = parse_container(tokens_in).unwrap();
        let token_stream = generate_container(model);

        let expected_tokens = quote! {
            impl IntoContainer for Nginx {
                fn into_container(self) -> Container {
                    let image = Image::from_name("docker.io/library/nginx");
                    let container = Container::from_image(image);
                    container
                }
            }
        };

        assert_eq!(token_stream.to_string(), expected_tokens.to_string());
    }

    #[test]
    fn test_generate_1() {
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
                #[env_var = "USER"]
                user: Option<String>
            }
        };

        let model = parse_container(tokens_in).unwrap();
        let token_stream = generate_container(model);

        let expected_tokens = quote! {
            impl IntoContainer for SimpleImage {
                fn into_container(self) -> Container {
                    let image = Image::from_name("docker.io/library/nginx");
                    let container = Container::from_image(image);
                    container.env_var(("PASSWORD", self.password));
                    self.user.and_then(|value| container.env_var(("USER", value)));
                    container.map_port((8080u32, 8080u32));
                    container.map_port((8081u32, 8080u32));
                    container
                }
            }
        };

        assert_eq!(token_stream.to_string(), expected_tokens.to_string());
    }
}
