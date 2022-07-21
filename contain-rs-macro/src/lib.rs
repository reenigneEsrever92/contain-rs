extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Attribute, DeriveInput, ExprAssign, ExprCall, ExprLit, Result};
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro_derive(Container, attributes(container, env_var))]
pub fn container_macro(item: TokenStream) -> TokenStream {
    parse_container(item.into()).into()
}

