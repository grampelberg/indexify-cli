use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

pub fn derive_parent(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;
    println!("{}", name);

    let target = match input.data {
        syn::Data::Struct(ref data) => data,
        _ => {
            return Err(Error::new_spanned(
                input,
                "Parent can only be derived for structs",
            ))
        }
    };

    // TODO: detect #[command(subcommand)]
    for field in target.fields.iter() {
        if has_attribute(field, "command") {
            println!("{:#?}", field.ty);
        }
    }

    // println!("{:#?}", input.data.fields);
    // for field in input.data.fields.iter() {
    //     println!("{:#?}", field);
    // }

    let expanded = quote! {
        #[automatically_derived]
        impl crate::command::Parent for #name {}
    };
    // let expanded = quote! {
    //     impl crate::command::Command for #name {
    //         fn next(&self) -> Option<&dyn crate::command::Command> {
    //             match self {
    //                 #name::Create(cmd) => Some(cmd),
    //                 #name::List(cmd) => Some(cmd),
    //                 #name::Get(cmd) => Some(cmd),
    //             }
    //         }
    //     }
    // };

    Ok(expanded)
}

fn has_attribute(field: &syn::Field, name: &str) -> bool {
    for attr in field.attrs.iter() {
        if attr.path().is_ident(name) {
            return true;
        }
    }

    false
}
