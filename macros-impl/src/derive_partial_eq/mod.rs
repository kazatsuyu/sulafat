mod r#enum;
mod r#struct;

use proc_macro2::{Span, TokenStream};
use r#enum::Enum;
use r#struct::Struct;
use syn::{parse2, Item};

pub fn derive_partial_eq_impl(items: TokenStream) -> syn::Result<TokenStream> {
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

pub fn derive_partial_eq(items: TokenStream) -> TokenStream {
    derive_partial_eq_impl(items).unwrap_or_else(|e| e.into_compile_error())
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::derive_partial_eq;

    #[test]
    fn it_works() {
        assert_eq!(
            derive_partial_eq(quote! {
                struct X {
                    a: i32,
                    b: i32,
                }
            })
            .to_string(),
            quote! {
                impl ::std::cmp::PartialEq for X {
                    fn eq(&self, other: &Self) -> bool {
                        ::std::cmp::PartialEq::eq(&self.a, &other.a)
                            && ::std::cmp::PartialEq::eq(&self.b, &other.b)
                            && true
                    }
                }
            }
            .to_string()
        )
    }
}
