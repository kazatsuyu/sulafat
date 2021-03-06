use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use syn::{
    Attribute, Fields, FieldsNamed, FieldsUnnamed, Generics, ItemStruct, Meta, MetaList, NestedMeta,
};

use crate::util::{fq_name::*, new_ident, Params};

pub(crate) struct Struct<'a> {
    item: &'a ItemStruct,
    params: Params<'a>,
    ident_str: Literal,
    s: Ident,
}

impl<'a> Struct<'a> {
    pub(crate) fn new(item: &'a ItemStruct) -> Self {
        let params = Params::from(&item.generics);
        let mut names = params
            .params
            .iter()
            .map(|param| param.to_string())
            .collect();
        let s = new_ident(&mut names, "S", &mut 0);
        Self {
            item,
            params,
            ident_str: Literal::string(&item.ident.to_string()),
            s,
        }
    }

    pub(crate) fn try_to_tokens(&self) -> syn::Result<TokenStream> {
        let Self {
            item:
                ItemStruct {
                    ident,
                    generics,
                    fields,
                    ..
                },
            params,
            s,
            ..
        } = self;
        let Generics { where_clause, .. } = generics;
        let (serializer, fields, end) = match fields {
            Fields::Unit => self.fields_unit(),
            Fields::Named(fields) => self.fields_named(fields),
            Fields::Unnamed(fields) => self.fields_unnamed(fields),
        };
        Ok(quote! {
            impl #generics #_Serialize for #ident #params #where_clause {
                fn serialize<#s>(&self, serializer: #s) -> #_Result<#s::Ok, #s::Error>
                where
                    #s: #_Serializer,
                {
                    #serializer #(#fields)* #end
                }
            }
        })
    }

    fn fields_unit(&self) -> (TokenStream, Vec<TokenStream>, TokenStream) {
        let Self { ident_str, .. } = self;
        (
            quote! {
                #_Serializer::serialize_unit_struct(serializer, #ident_str)
            },
            vec![],
            quote! {},
        )
    }

    fn fields_named(&self, fields: &FieldsNamed) -> (TokenStream, Vec<TokenStream>, TokenStream) {
        let Self { ident_str, .. } = self;
        let fields = fields
            .named
            .iter()
            .filter_map(|field| {
                if Self::is_skip(&field.attrs) {
                    return None;
                }
                let ident = field.ident.as_ref().unwrap();
                let ident_str = Literal::string(&ident.to_string());
                Some(quote! {
                    #_SerializeStruct::serialize_field(&mut serializer, #ident_str, &self.#ident)?;
                })
            })
            .collect::<Vec<_>>();
        let len = Literal::usize_unsuffixed(fields.len());
        let serializer = quote! {
            use #_SerializeStruct as _;
            let mut serializer = #_Serializer::serialize_struct(serializer, #ident_str, #len)?;
        };
        let end = quote! {
            #_SerializeStruct::end(serializer)
        };
        (serializer, fields, end)
    }

    fn fields_unnamed(
        &self,
        fields: &FieldsUnnamed,
    ) -> (TokenStream, Vec<TokenStream>, TokenStream) {
        let Self { ident_str, .. } = self;
        let len = Literal::usize_unsuffixed(fields.unnamed.len());
        let serializer = quote! {
            let mut serializer = #_Serializer::serialize_tuple_struct(serializer, #ident_str, #len)?;
        };
        let fields = (0..fields.unnamed.len())
            .map(|index| {
                let index = Literal::usize_unsuffixed(index);
                quote! {
                    #_SerializeTupleStruct::serialize_field(&mut serializer, &self.#index)?;
                }
            })
            .collect();
        let end = quote! {
            #_SerializeTupleStruct::end(serializer)
        };
        (serializer, fields, end)
    }

    fn is_skip(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            if let Ok(Meta::List(MetaList { path, nested, .. })) = attr.parse_meta() {
                if path.leading_colon.is_none()
                    && path.segments.len() == 1
                    && path.segments[0].arguments.is_empty()
                    && path.segments[0].ident.to_string() == "serde"
                {
                    nested.into_iter().any(|nested_meta| {
                        if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                            path.leading_colon.is_none()
                                && path.segments.len() == 1
                                && path.segments[0].arguments.is_empty()
                                && path.segments[0].ident.to_string() == "skip"
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            } else {
                false
            }
        })
    }
}
