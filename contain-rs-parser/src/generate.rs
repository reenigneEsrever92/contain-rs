use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::model::{Command, FieldAttribute, HealthCheck, Model, ModelField, WaitLog, WaitTime};

pub fn generate_container(model: Model) -> TokenStream {
    let struct_name = format_ident!("{}", model.struct_name);
    let image_name = model.image;
    let fields = model.fields;
    let health_check = model.health_check.iter();
    let command = model.command;
    let wait_time = model.wait_time;
    let wait_log = model.wait_log;

    quote! {
        impl IntoContainer for #struct_name {
            fn into_container(self) -> Container {
                use std::str::FromStr;
                use std::time::Duration;
                use contain_rs::*;

                let image = Image::from_str(#image_name).unwrap();
                let mut container = Container::from_image(image);
                #command
                #( #fields )*
                #( #health_check )*
                #wait_time
                #wait_log
                container
            }
        }
    }
}

impl ToTokens for WaitLog {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let log_message = &self.message;
        tokens.extend(quote! {
            container.wait_for(WaitStrategy::LogMessage { pattern: Regex::new(#log_message).unwrap() });
        })
    }
}

impl ToTokens for WaitTime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let millis = self.time.as_millis() as u64;
        tokens.extend(quote! {
            container.wait_for(WaitStrategy::WaitTime { duration: Duration::from_millis(#millis) });
        })
    }
}

impl ToTokens for Command {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let command = &self.args;

        tokens.extend(quote! { container.command(vec![#(#command.to_string(),)*]); });
    }
}

impl ToTokens for HealthCheck {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            HealthCheck::Command(command) => tokens.extend(quote! {
                container.health_check(HealthCheck::new(#command))
                    .wait_for(WaitStrategy::HealthCheck);
            }),
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

fn generate_field_tokens(field: &ModelField, attributes: &[FieldAttribute]) -> Vec<TokenStream> {
    attributes
        .iter()
        .map(|attr| match attr {
            FieldAttribute::EnvVar(name) => generate_env_var(field, name),
            FieldAttribute::Arg(name) => generate_arg(field, name),
            FieldAttribute::Port(port) => generate_port(field, *port),
        })
        .collect()
}

fn generate_port(field: &ModelField, port: u32) -> TokenStream {
    let field_name = format_ident!("{}", &field.name);

    match field.r#type {
        crate::model::FieldType::Simple => quote! {
            container.map_port(&self.#field_name, #port);
        },
        crate::model::FieldType::Option => quote! {
            if let Some(value) = self.#field_name {
                container.map_port(&value, #port);
            }
        },
    }
}

fn generate_arg(field: &ModelField, name: &str) -> TokenStream {
    let field_name = format_ident!("{}", &field.name);

    match field.r#type {
        crate::model::FieldType::Simple => quote! {
            container.arg(#name);
            container.arg(&self.#field_name);
        },
        crate::model::FieldType::Option => quote! {
            if let Some(value) = self.#field_name {
                container.arg(#name);
                container.arg(&value);
            }
        },
    }
}

fn generate_env_var(field: &ModelField, name: &str) -> TokenStream {
    let field_name = format_ident!("{}", &field.name);

    match field.r#type {
        crate::model::FieldType::Simple => quote! {
            container.env_var(#name, &self.#field_name);
        },
        crate::model::FieldType::Option => quote! {
            if let Some(value) = self.#field_name {
                container.env_var(#name, &value);
            }
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
                    use std::str::FromStr;
                    use std::time::Duration;
                    use contain_rs::*;

                    let image = Image::from_str("docker.io/library/nginx").unwrap();
                    let mut container = Container::from_image(image);
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
                command = ["nginx", "-g", "daemon off;"],
                health_check_command = "curl http://localhost || exit 1",
                health_check_timeout = 30000,
                wait_time = 1000,
                wait_log = "test"
            )]
            struct SimpleImage {
                #[contain_rs(env_var = "PASSWORD")]
                password: String,
                #[contain_rs(env_var = "USER")]
                user: Option<String>,
                #[contain_rs(arg = "--arg")]
                arg: String,
                #[contain_rs(port = 8080)]
                web_port: u32,
            }
        };

        let model = parse_container(tokens_in).unwrap();
        let token_stream = generate_container(model);

        let expected_tokens = quote! {
            impl IntoContainer for SimpleImage {
                fn into_container(self) -> Container {
                    use std::str::FromStr;
                    use std::time::Duration;
                    use contain_rs::*;

                    let image = Image::from_str("docker.io/library/nginx").unwrap();
                    let mut container = Container::from_image(image);
                    container.command(vec!["nginx".to_string(), "-g".to_string(), "daemon off;".to_string(),]);
                    container.env_var("PASSWORD", &self.password);
                    if let Some(value) = self.user {
                        container.env_var("USER", &value);
                    }
                    container.arg("--arg");
                    container.arg(&self.arg);
                    container.map_port(&self.web_port, 8080u32);
                    container.health_check(HealthCheck::new("curl http://localhost || exit 1"))
                        .wait_for(WaitStrategy::HealthCheck);
                    container.wait_for(WaitStrategy::WaitTime { duration: Duration::from_millis(1000u64) });
                    container.wait_for(WaitStrategy::LogMessage { pattern: Regex::new("test").unwrap() });
                    container
                }
            }
        };

        assert_eq!(token_stream.to_string(), expected_tokens.to_string());
    }
}
