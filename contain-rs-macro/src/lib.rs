extern crate proc_macro;

use contain_rs_parser::container;
use proc_macro::TokenStream;

#[proc_macro_derive(ContainerImpl, attributes(container, env_var, arg))]
pub fn container_macro(item: TokenStream) -> TokenStream {
    container(item.into()).unwrap().into()
}
