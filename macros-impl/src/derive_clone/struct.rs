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
        let (clone_fields, clone_from_fields) = match fields {
            Fields::Unit => (quote! { Self }, quote! {}),
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();
                let clone = _Clone;
                (
                    quote! {
                        Self {
                            #(
                                #fields: #clone::clone(&self.#fields),
                            )*
                        }
                    },
                    quote! {
                        #(
                            #clone::clone_from(&mut self.#fields, &source.#fields);
                        )*
                    },
                )
            }
            Fields::Unnamed(fields) => {
                let fields = (0..fields.unnamed.len())
                    .map(|field| Index::from(field))
                    .collect::<Vec<_>>();
                let clone = _Clone;
                (
                    quote! {
                        Self(#(#clone::clone(&self.#fields)),*)
                    },
                    quote! {
                        #(
                            #clone::clone_from(&mut self.#fields, &source.#fields);
                        )*
                    },
                )
            }
        };
        Ok(quote! {
            impl #generics #_Clone for #ident #params #where_clause {
                fn clone(&self) -> Self {
                    #clone_fields
                }
                fn clone_from(&mut self, source: &Self) {
                    #clone_from_fields
                }
            }
        })
    }
}
