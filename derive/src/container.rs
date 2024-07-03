use eyre::Result;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::visit::{self, Visit};

// TODO: would this be better as a visitor?
fn get_field_name(data: &syn::DataStruct, attr_name: &str) -> Option<Ident> {
    for field in data.fields.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident(attr_name) {
                return field.ident.clone();
            }
        }
    }

    None
}

fn struct_impl(name: &Ident, data: &syn::DataStruct) -> Result<TokenStream, syn::Error> {
    let next_impl = match get_field_name(data, "command") {
        Some(field_name) => quote! {
            fn next(&self) -> Option<&dyn crate::command::Command> {
                self.#field_name.next()
            }
        },
        None => quote! {
             fn next(&self) -> Option<&dyn crate::command::Command> {
                 None
             }
        },
    };

    Ok(quote! {
        #[automatically_derived]
        impl crate::command::Container for #name {
            #next_impl
        }
    })
}

#[derive(Default)]
struct UnnamedTypes<'ast> {
    commands: Vec<&'ast syn::Ident>,
}

impl<'ast> Visit<'ast> for UnnamedTypes<'ast> {
    fn visit_variant(&mut self, i: &'ast syn::Variant) {
        self.commands.push(&i.ident);

        visit::visit_variant(self, i);
    }
}

fn enum_impl(name: &Ident, data: &syn::DataEnum) -> Result<TokenStream, syn::Error> {
    let mut visitor = UnnamedTypes::default();
    visitor.visit_data_enum(data);

    let commands = visitor.commands;

    Ok(quote! {
        #[automatically_derived]
        impl crate::command::Container for #name {
            fn next(&self) -> Option<&dyn crate::command::Command> {
                match self {
                    #(Self::#commands(cmd) => Some(cmd),)*
                }
            }
        }
    })
}

pub fn derive_container(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;

    match input.data {
        syn::Data::Struct(ref data) => struct_impl(name, data),
        syn::Data::Enum(ref data) => enum_impl(name, data),
        _ => Err(syn::Error::new_spanned(
            input,
            "Command can only be derived for structs or enums",
        )),
    }
}
