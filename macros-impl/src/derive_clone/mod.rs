mod r#enum;
mod r#struct;

use proc_macro2::{Span, TokenStream};
use r#enum::Enum;
use r#struct::Struct;
use syn::{parse2, Item};

pub fn derive_clone_impl(items: TokenStream) -> syn::Result<TokenStream> {
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

pub fn derive_clone(items: TokenStream) -> TokenStream {
    derive_clone_impl(items).unwrap_or_else(|e| e.into_compile_error())
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::derive_clone;

    #[test]
    fn it_works() {
        assert_eq!(
            derive_clone(quote! {
                enum X {
                    A,
                    B,
                }
            })
            .to_string(),
            quote! {
                impl ::std::clone::Clone for X {
                    fn clone(&self) -> Self {
                        match self {
                            X::A => X::A,
                            X::B => X::B,
                        }
                    }
                }
            }
            .to_string()
        )
    }
}
