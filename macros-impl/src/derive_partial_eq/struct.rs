use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, Generics, Index, ItemStruct};

use crate::util::{fq_name::*, Params};

pub(crate) struct Struct<'a> {
    item: &'a ItemStruct,
    params: Params<'a>,
}

impl<'a> Struct<'a> {
    pub(crate) fn new(item: &'a ItemStruct) -> Self {
        let ItemStruct { generics, .. } = item;
        let params = Params::from(generics);
        Self { item, params }
    }
    pub(crate) fn try_to_tokens(&self) -> syn::Result<TokenStream> {
        let Struct {
            item:
                ItemStruct {
                    ident,
                    generics,
                    fields,
                    ..
                },
            params,
        } = self;
        let Generics { where_clause, .. } = generics;
        let fields = match fields {
            Fields::Unit => {
                quote! { true }
            }
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());
                let partial_eq = _PartialEq;
                quote! {
                    #(#partial_eq::eq(&self.#fields, &other.#fields) &&)* true
                }
            }
            Fields::Unnamed(fields) => {
                let fields = (0..fields.unnamed.len()).map(|field| Index::from(field));
                let partial_eq = _PartialEq;
                quote! {
                    #(#partial_eq::eq(&self.#fields, &other.#fields) &&)* true
                }
            }
        };
        Ok(quote! {
            impl #generics #_PartialEq for #ident #params #where_clause {
                fn eq(&self, other: &Self) -> bool {
                    #fields
                }
            }
        })
    }
}
