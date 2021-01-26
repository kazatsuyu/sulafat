use std::collections::HashSet;

use crate::util::{fq_name::*, new_ident, Params};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ConstParam, Fields, GenericParam, Generics, ItemEnum, TypeParam};

pub(crate) struct Enum<'a> {
    item: &'a ItemEnum,
    params: Params<'a>,
}

impl<'a> Enum<'a> {
    pub(crate) fn new(item: &'a ItemEnum) -> Self {
        let ItemEnum { generics, .. } = item;
        let params = Params::from(generics);
        Self { item, params }
    }

    pub(crate) fn try_to_tokens(&self) -> syn::Result<TokenStream> {
        let Enum {
            item:
                ItemEnum {
                    ident,
                    generics,
                    variants,
                    ..
                },
            params,
        } = self;
        let Generics { where_clause, .. } = generics;
        let mut names = variants
            .iter()
            .map(|variant| variant.ident.to_string())
            .collect::<HashSet<_>>();
        names.extend(generics.params.iter().filter_map(|param| match param {
            GenericParam::Const(ConstParam { ident, .. })
            | GenericParam::Type(TypeParam { ident, .. }) => Some(ident.to_string()),
            GenericParam::Lifetime(..) => None,
        }));
        let variants = variants.iter().map(|variant| {
            let variant_ident = &variant.ident;
            match &variant.fields {
                Fields::Unit => {
                    quote! { #ident::#variant_ident => #ident::#variant_ident, }
                }
                Fields::Named(fields) => {
                    let fields = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();
                    let mut i = 0;
                    let self_fields = (0..fields.len())
                        .map(|_| new_ident(&mut names, "self_field", &mut i))
                        .collect::<Vec<_>>();
                    let clone = _Clone;
                    quote! {
                        #ident::#variant_ident {
                            #(#fields: #self_fields,)*
                        } => #ident::#variant_ident {
                            #(#fields: #clone::clone(#self_fields),)*
                        },
                    }
                }
                Fields::Unnamed(fields) => {
                    let mut i = 0;
                    let self_fields = (0..fields.unnamed.len())
                        .map(|_| new_ident(&mut names, "self_field", &mut i))
                        .collect::<Vec<_>>();
                    let clone = _Clone;
                    quote! {
                        #ident::#variant_ident (
                            #(#self_fields,)*
                        ) => #ident::#variant_ident (
                            #(#clone::clone(#self_fields),)*
                        ),
                    }
                }
            }
        });
        Ok(quote! {
            impl #generics #_Clone for #ident #params #where_clause {
                fn clone(&self) -> Self {
                    match self  {
                        #(#variants)*
                    }
                }
            }
        })
    }
}
