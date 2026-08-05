use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(LogDifferences)]
pub fn derive_macro_log_differences(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate.
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;

    let data = if let syn::Data::Struct(data) = ast.data {
        data
    } else {
        unimplemented!();
    };

    let fields = data.fields.iter().map(|f| {
        return if let Some(ident) = &f.ident {
            let ident_str = ident.to_string();
            quote! {
                crate::differ::LogDifferences::log_differences(
                    &a.#ident,
                    &b.#ident,
                    format!("{}.{}", context, #ident_str)
                );
            }
        } else {
            quote! {}
        };
    });

    let expanded = quote! (
        impl crate::differ::LogDifferences for #name {
            fn log_differences(a: &Self, b: &Self, context: String) {
                #(#fields)*
            }
        }
    );

    return expanded.into();
}
