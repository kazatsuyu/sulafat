mod r#enum;
mod r#struct;
use proc_macro2::{Span, TokenStream};
use r#enum::Enum;
use r#struct::Struct;
use syn::{parse2, Item};

fn derive_serialize_impl(items: TokenStream) -> syn::Result<TokenStream> {
    let item = parse2::<Item>(items)?;
    match item {
        Item::Struct(item) => Struct::new(&item).try_to_tokens(),
        Item::Enum(item) => Enum::new(&item).try_to_tokens(),
        _ => Err(syn::Error::new(
            Span::call_site(),
            "Struct or enum is expected.",
        )),
    }
}

pub fn derive_serialize(items: TokenStream) -> TokenStream {
    derive_serialize_impl(items).unwrap_or_else(|e| e.to_compile_error())
}
