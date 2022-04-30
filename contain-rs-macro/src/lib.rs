extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, DeriveInput, ExprAssign, ExprLit, Lit,
    Token,
};

#[proc_macro_attribute]
pub fn container(args: TokenStream, item: TokenStream) -> TokenStream {
    println!("Image Attribute: \n\n");

    println!("ARGS INPUT: {}", &args);
    println!("ITEM INPUT: {}", item);

    let args_ast: MacroInput = syn::parse(args).unwrap();
    let item_ast: DeriveInput = syn::parse(item).unwrap();

    println!("ARGS AST: {:#?}", args_ast);
    println!("ITEM AST: {:#?}", item_ast);

    let it = parse(&args_ast).unwrap();

    let result = generate_into_container(&item_ast, &it);

    println!("OUTPUT: {}", result);

    result
    // TokenStream::new()
}

#[derive(Debug)]
struct MacroInput {
    attrs: Punctuated<ExprAssign, Token!(,)>,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = Punctuated::new();

        loop {
            if input.is_empty() {
                break;
            };

            attrs.push_value(input.parse()?);

            if input.is_empty() {
                break;
            }

            attrs.push_punct(input.parse()?);
        }

        Ok(MacroInput { attrs })
    }
}

#[allow(dead_code)]
enum ContainerParam {
    Image(String),
    HealthCheck(String),
}

fn parse(args: &MacroInput) -> syn::Result<Vec<ContainerParam>> {
    let mapped: Result<Vec<ContainerParam>, syn::Error> = args
        .attrs
        .iter()
        .map(|expr| match &*expr.left {
            syn::Expr::Path(p) => {
                if p.path.is_ident("image") {
                    match &*expr.right {
                        syn::Expr::Lit(l) => return map_to_image(l),
                        _ => return Err(syn::Error::new(expr.right.span(), "Expected Literal")),
                    }
                }

                Err(syn::Error::new(expr.left.span(), "Unknown Image Attribute"))
            }
            _ => Err(syn::Error::new(
                expr.left.span(),
                "Expected Path Expression",
            )),
        })
        .collect();

    mapped
}

fn map_to_image(literal: &ExprLit) -> syn::Result<ContainerParam> {
    match &literal.lit {
        Lit::Str(str) => Ok(ContainerParam::Image(str.value())),
        _ => Err(syn::Error::new(literal.span(), "Expected String Literal")),
    }
}

fn generate_into_container(
    item: &syn::DeriveInput,
    container_params: &Vec<ContainerParam>,
) -> TokenStream {
    let ident = &item.ident; 
    let image_name = find_image(container_params).unwrap();

    println!("Ident: {}", ident);

    quote! {
        #item

        impl #ident {
            fn new() -> Self {
                Self {}
            }
        }

        impl IntoContainer for #ident {
            fn into_container(self) -> Container {
                Container::from_image(Image::from_name(#image_name))
            }
        }
    }
    .into()
}

fn find_image(container_params: &Vec<ContainerParam>) -> Option<String> {
    container_params.iter().find(|it| match it {
        ContainerParam::Image(_) => true,
        _ => false,
    }).map(|it| {
        match it {
            ContainerParam::Image(name) => name.to_owned(),
            _ => panic!(),
        }
    })
}
