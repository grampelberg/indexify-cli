mod container;

use proc_macro::TokenStream;

#[proc_macro_derive(Command)]
pub fn derive_container(input: TokenStream) -> TokenStream {
    container::derive_container(syn::parse_macro_input!(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
