use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2, Fields, Ident, ItemEnum};

use crate::util::{crate_name, Params};

fn derive_variant_ident_impl(items: TokenStream) -> syn::Result<TokenStream> {
    let item_enum = parse2::<ItemEnum>(items)?;
    let vis = &item_enum.vis;
    let types_ident = Ident::new(
        &format!("{}VariantIdent", item_enum.ident),
        Span::call_site(),
    );
    let variants = item_enum
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .collect::<Vec<_>>();
    let ident = &item_enum.ident;
    let match_arms = item_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            match &variant.fields {
                Fields::Unit => quote! { #ident::#variant_ident => #types_ident::#variant_ident, },
                Fields::Unnamed(..) => {
                    quote! { #ident::#variant_ident(..) => #types_ident::#variant_ident, }
                }
                Fields::Named(..) => {
                    quote! { #ident::#variant_ident{..} => #types_ident::#variant_ident, }
                }
            }
        })
        .collect::<Vec<_>>();
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let params = Params::from(generics);
    let sulafat_vdom = crate_name("sulafat-vdom");
    let mut from = None;
    for attr in &item_enum.attrs {
        let path = &attr.path;
        if path.leading_colon.is_none()
            && path.segments.len() == 1
            && path.segments[0].ident == "from"
        {
            let from_tokens = attr.parse_args::<TokenStream>()?;
            from = Some(quote! {
                impl From<#from_tokens> for #types_ident {
                    fn from(src: #from_tokens) -> Self {
                        match src {
                            #(#from_tokens::#variants => #types_ident::#variants,)*
                        }
                    }
                }
            });
            break;
        }
    }
    Ok(quote! {
        const _: () = {
            #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, ::serde_derive::Serialize, ::serde_derive::Deserialize)]
            #vis enum #types_ident {
                #(#variants,)*
            }
            impl #generics ::#sulafat_vdom::VariantIdent for #ident #params #where_clause {
                type Type = #types_ident;
                fn variant_ident(&self) -> Self::Type {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
            #from
        };
    })
}

pub fn derive_variant_ident(items: TokenStream) -> TokenStream {
    derive_variant_ident_impl(items).unwrap_or_else(|e| e.to_compile_error())
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::derive_variant_ident;

    #[test]
    fn it_works() {
        assert_eq!(
            derive_variant_ident(quote! {
                pub enum Element<Msg> {
                    Div(Div<Msg>),
                    Span(Span<Msg>),
                }
            })
            .to_string(),
            quote! {
                const _: () = {
                    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, ::serde_derive::Serialize, ::serde_derive::Deserialize)]
                    pub enum ElementVariantIdent {
                        Div,
                        Span,
                    }
                    impl<Msg> ::sulafat_vdom::VariantIdent for Element<Msg> {
                        type Type = ElementVariantIdent;
                        fn variant_ident(&self) -> Self::Type {
                            match self {
                                Element::Div(..) => ElementVariantIdent::Div,
                                Element::Span(..) => ElementVariantIdent::Span,
                            }
                        }
                    }
                };
            }
            .to_string()
        )
    }
}
