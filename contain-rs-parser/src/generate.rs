use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::model::{
    Command, FieldAttribute, HealthCheck, Model, ModelField, Port, WaitLog, WaitTime,
};

pub fn generate_container(model: Model) -> TokenStream {
    let struct_name = format_ident!("{}", model.struct_name);
    let image_name = model.image;
    let fields = model.fields;
    let ports = model.ports;
    let health_check = model.health_check.iter();
    let command = model.command;
    let wait_time = model.wait_time;
    let wait_log = model.wait_log;

    // TODO generate wait time

    quote! {
        impl IntoContainer for #struct_name {
            fn into_container(self) -> Container {
                let image = Image::from_str(#image_name).unwrap();
                let mut container = Container::from_image(image);
                #command
                #( #fields )*
                #( #ports )*
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

impl ToTokens for Port {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let source = self.source;
        let target = self.target;
        tokens.extend(quote! { container.map_port(#source, #target); })
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
                ports = [8080:8080, 8081:8080],
                wait_time = 1000,
                wait_log = "test"
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
                    let image = Image::from_str("docker.io/library/nginx").unwrap();
                    let mut container = Container::from_image(image);
                    container.command(vec!["nginx".to_string(), "-g".to_string(), "daemon off;".to_string(),]);
                    container.env_var(("PASSWORD", self.password));
                    self.user.and_then(|value| container.env_var(("USER", value)));
                    container.map_port(8080u32, 8080u32);
                    container.map_port(8081u32, 8080u32);
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
