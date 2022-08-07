use syn::{Result, DeriveInput};

use crate::parse::Model;

#[derive(Debug)]
pub struct IR {
    pub struct_name: String,
    pub image: String,
    pub env_vars: Vec<(String, String)>,
}

pub fn parse(model: &Model) -> Result<IR> {
    todo!()
}

fn get_struct_name(ast: &DeriveInput) -> Result<String> {
    Ok(ast.ident.to_string())
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::ir::get_struct_name;

    #[test]
    fn test_get_struct_name() {
        let tokens: proc_macro2::TokenStream = quote! {
            #[derive(Container)]
            struct TestStruct;
        };

        let derive_input = syn::parse2(tokens).unwrap();

        assert_eq!(
            get_struct_name(&derive_input).unwrap(),
            "TestStruct".to_string()
        );
    }
}