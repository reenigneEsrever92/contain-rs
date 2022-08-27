use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::model::{FieldAttribute, Model, ModelField};

pub fn generate_container(model: Model) -> TokenStream {
    let struct_name = model.struct_name;
    let image_name = model.image;
    let fields = model.fields;

    quote! {
        impl IntoContainer for #struct_name {
            fn from(value: #struct_name) -> Container {
                let image = Image::from_name(#image_name);
                let container = Container::from_image(image);
                #( #fields )*
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

fn generate_field_tokens(field: &ModelField, attributes: &[FieldAttribute]) -> Vec<TokenStream> {
    attributes
        .iter()
        .map(|attr| match attr {
            FieldAttribute::EnvVar(name) => generate_env_var(field, name),
        })
        .collect()
}

fn generate_env_var(field: &ModelField, name: &str) -> TokenStream {
    let field_name = &field.name;
    quote! {
        container.env_var((#name, #field_name));
    }
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::{generate::generate_container, parse::parse_container};

    #[test]
    fn test_generate() {
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

        let model = parse_container(tokens_in).unwrap();
        let token_stream = generate_container(model);

        let expected_tokens = quote! {
            impl IntoContainer for SimpleImage {
                fn from(value: SimpleImage) -> Container {
                    let image = Image::from_name("docker.io/library/nginx");
                    let container = Container::from_image(image);
                    container.env_var("PASSWORD", self.password);
                    container
                }
            }
        };

        assert_ne!(token_stream.to_string(), expected_tokens.to_string());
    }
}
