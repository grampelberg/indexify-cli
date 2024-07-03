use eyre::Result;
use proc_macro2::TokenStream;
use quote::quote;

pub fn derive_info(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;

    let expanded = quote! {
        #[automatically_derived]
        impl crate::command::Info for #name {
            fn command(&self) -> clap::Command {
                <Self as clap::CommandFactory>::command()
            }
        }
    };

    Ok(expanded.into())
}
