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
