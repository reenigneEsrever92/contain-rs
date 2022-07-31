extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Container, attributes(container, env_var))]
pub fn container_macro(item: TokenStream) -> TokenStream {
    item
}
