mod generate;
mod model;
mod parse;

use proc_macro2::TokenStream as TokenStream2;

use syn::Result as SynResult;

use crate::{generate::generate_container, parse::parse_container};

pub fn container(tokens: TokenStream2) -> SynResult<TokenStream2> {
    let model = parse_container(tokens);
    Ok(generate_container(model?))
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::{
        model::{FieldAttribute, FieldType, HealthCheck, Model, ModelField},
        parse::parse_container,
    };

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
                command: None,
                struct_name: "SimpleImage".to_string(),
                image: "docker.io/library/nginx".to_string(),
                health_check: Some(HealthCheck::Command(
                    "curl http://localhost || exit 1".to_string()
                )),
                ports: Vec::new(),
                fields: vec![ModelField {
                    name: "password".to_string(),
                    ty: FieldType::Simple,
                    attributes: vec![FieldAttribute::EnvVar("PASSWORD".to_string())]
                }]
            }
        );
    }
}
