use proc_macro2::TokenStream;
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, DeriveInput, ExprAssign, Field, Result,
    Token,
};

#[derive(Debug)]
pub struct Model {
    pub derive_input: DeriveInput,
    pub attrs: AttributeInput,
    pub fields: Vec<FieldModel>,
}

#[derive(Debug)]
pub struct FieldModel {
    pub field: Field,
    pub attrs: AttributeInput,
}

#[derive(Debug)]
pub struct AttributeInput {
    pub vars: Punctuated<ExprAssign, Token![,]>,
}

pub fn parse(input: TokenStream) -> Result<Model> {
    let derive_input: DeriveInput = syn::parse2(input)?;

    println!("PARSED DERIVE INPUT: {:#?}", &derive_input);

    let tokens = get_container_attributes(&derive_input)?;

    println!("ATTRIBUTES: {:#?}", &tokens);

    // let attrs: AttributeInput = syn::parse2(tokens)?;

    // println!("PARSED ATTRIBUTES: {:#?}", &attrs);    

    Ok(Model {
        derive_input,
        attrs: AttributeInput {
            vars: Punctuated::new(),
        },
        fields: Vec::new(),
    })
}

fn get_container_attributes<'a>(input: &'a DeriveInput) -> Result<proc_macro2::TokenStream> {
    let attrs = input.attrs.iter().fold(None, |left, right| match left {
        Some(thing) => match thing {
            Ok(_) => Some(Err(syn::Error::new(
                right.span(),
                "Expected only one container annotation",
            ))),
            Err(mut e) => {
                e.combine(syn::Error::new(
                    right.span(),
                    "Expected only one container annotation",
                ));

                Some(Err(e))
            }
        },
        None => Some(Ok(right)),
    });

    match attrs {
        Some(result) => result.map(|it| it.tokens.clone()),
        None => Err(syn::Error::new(
            input.span(),
            "Expected container annotation",
        )),
    }
}

impl Parse for AttributeInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut punctuated = Punctuated::new();

        // loop {
        //     if input.lookahead1().peek(syn::Ident) {
        //         punctuated.push_value(input.parse()?);

        //         if input.lookahead1().peek(syn::token::Comma) {
        //             punctuated.push_punct(input.parse()?);
        //         }
        //     } else {
        //         break;
        //     }
        // }

        Ok(Self { vars: punctuated })
    }
}
