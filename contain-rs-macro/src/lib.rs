extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Attribute, DeriveInput, ExprAssign, ExprCall, ExprLit, Result};

#[proc_macro_derive(Container, attributes(container, env_var))]
pub fn container_macro(item: TokenStream) -> TokenStream {
    todo!()
}
