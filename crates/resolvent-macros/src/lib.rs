use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result, Token, Visibility, parse_macro_input};

mod kw {
    syn::custom_keyword!(model);
}

struct PhysicsInput {
    vis: Visibility,
    name: Ident,
    source: LitStr,
}

impl Parse for PhysicsInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let vis = input.parse()?;
        input.parse::<kw::model>()?;
        let name = input.parse()?;
        input.parse::<Token![=]>()?;
        let source = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { vis, name, source })
    }
}

#[proc_macro]
pub fn physics(input: TokenStream) -> TokenStream {
    let PhysicsInput { vis, name, source } = parse_macro_input!(input as PhysicsInput);
    quote! {
        #[derive(Clone, Copy, Debug, Default)]
        #vis struct #name;

        impl #name {
            #vis const SOURCE: &'static str = #source;

            #vis fn parse() -> Result<::resolvent::author::ParsedModel, ::resolvent::author::AuthorError> {
                ::resolvent::author::parse_model(Self::SOURCE)
            }

            #vis fn elaborate() -> Result<::resolvent::author::ElaboratedModel, ::resolvent::author::AuthorError> {
                ::resolvent::author::elaborate(Self::SOURCE)
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn include_physics(input: TokenStream) -> TokenStream {
    let PhysicsInput { vis, name, source } = parse_macro_input!(input as PhysicsInput);
    quote! {
        #[derive(Clone, Copy, Debug, Default)]
        #vis struct #name;

        impl #name {
            #vis const SOURCE: &'static str = include_str!(#source);

            #vis fn parse() -> Result<::resolvent::author::ParsedModel, ::resolvent::author::AuthorError> {
                ::resolvent::author::parse_model(Self::SOURCE)
            }

            #vis fn elaborate() -> Result<::resolvent::author::ElaboratedModel, ::resolvent::author::AuthorError> {
                ::resolvent::author::elaborate(Self::SOURCE)
            }
        }
    }
    .into()
}
