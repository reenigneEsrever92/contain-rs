use quote::quote;

use crate::ir::IR;

pub fn generate_into_container(model: &IR) -> proc_macro2::TokenStream {
    let ident = &model.struct_name;

    quote! {
        // impl IntoContainer for #ident {
        //     fn into_container(self) -> Container {
        //         #container_builder.into_container()
        //     }
        // }
    }
    .into()
}