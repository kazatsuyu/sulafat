use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::{Fields, FieldsNamed, FieldsUnnamed, Generics, ItemEnum, Variant};

use crate::util::{fq_name::*, new_ident, Params};

pub(crate) struct Enum<'a> {
    item: &'a ItemEnum,
    params: Params<'a>,
    ident_str: Literal,
    s: Ident,
}

impl<'a> Enum<'a> {
    pub(crate) fn new(item: &'a ItemEnum) -> Self {
        let params = Params::from(&item.generics);
        let mut names = params
            .params
            .iter()
            .map(|param| param.to_string())
            .collect();
        let s = new_ident(&mut names, "S", &mut 0);
        Self {
            item,
            params: Params::from(&item.generics),
            ident_str: Literal::string(&item.ident.to_string()),
            s,
        }
    }

    pub(crate) fn try_to_tokens(&self) -> syn::Result<TokenStream> {
        let Self {
            item:
                ItemEnum {
                    ident,
                    generics,
                    variants,
                    ..
                },
            params,
            s,
            ..
        } = self;
        let Generics { where_clause, .. } = generics;
        let variants = variants
            .into_iter()
            .enumerate()
            .map(
                |(
                    index,
                    Variant {
                        ident: variant_ident,
                        fields,
                        ..
                    },
                )| {
                    let variant_ident_str = Literal::string(&variant_ident.to_string());
                    let (pat, serializer, fields, end) = match fields {
                        Fields::Unit => self.variant_unit(index, variant_ident, &variant_ident_str),
                        Fields::Named(fields) => {
                            self.variant_named(index, fields, variant_ident, &variant_ident_str)
                        }
                        Fields::Unnamed(fields) => {
                            self.variant_unnamed(index, fields, variant_ident, &variant_ident_str)
                        }
                    };
                    quote! {
                        #pat => {
                            #serializer #(#fields)* #end
                        }
                    }
                },
            )
            .collect::<Vec<_>>();
        Ok(quote! {
            impl #generics #_Serialize for #ident #params #where_clause {
                fn serialize<#s>(&self, serializer: #s) -> #_Result<#s::Ok, #s::Error>
                where
                    #s: #_Serializer,
                {
                    match self {
                        #(#variants)*
                    }
                }
            }
        })
    }

    fn variant_unit(
        &self,
        index: usize,
        variant_ident: &Ident,
        variant_ident_str: &Literal,
    ) -> (TokenStream, TokenStream, Vec<TokenStream>, TokenStream) {
        let Self {
            item: ItemEnum { ident, .. },
            ident_str,
            ..
        } = self;
        let index = Literal::usize_unsuffixed(index);
        let pat = quote! {
            #ident::#variant_ident
        };
        let serialize = quote! {
            #_Serializer::serialize_unit_variant(serializer, #ident_str, #index, #variant_ident_str)
        };
        let fields = vec![];
        let end = quote! {};
        (pat, serialize, fields, end)
    }

    fn variant_named(
        &self,
        index: usize,
        fields: &FieldsNamed,
        variant_ident: &Ident,
        variant_ident_str: &Literal,
    ) -> (TokenStream, TokenStream, Vec<TokenStream>, TokenStream) {
        let Self {
            item: ItemEnum { ident, .. },
            ident_str,
            ..
        } = self;
        let index = Literal::usize_unsuffixed(index);
        let len = Literal::usize_unsuffixed(fields.named.len());
        let named = fields
            .named
            .iter()
            .map(|field| field.ident.clone().unwrap())
            .collect::<Vec<_>>();
        let pat = quote! {
            #ident::#variant_ident { #(#named),* }
        };
        let serialize = quote! {
            let mut serializer = #_Serializer::serialize_struct_variant(serializer, #ident_str, #index, #variant_ident_str, #len)?;
        };
        let fields = fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap();
                quote! {
                    #_SerializeStructVariant::serialize_struct_variant(&mut serializer, &#ident)?;
                }
            })
            .collect();
        let end = quote! {
            #_SerializeStructVariant::end(serializer)
        };
        (pat, serialize, fields, end)
    }

    fn variant_unnamed(
        &self,
        index: usize,
        fields: &FieldsUnnamed,
        variant_ident: &Ident,
        variant_ident_str: &Literal,
    ) -> (TokenStream, TokenStream, Vec<TokenStream>, TokenStream) {
        let Self {
            item: ItemEnum { ident, .. },
            ident_str,
            ..
        } = self;
        let index = Literal::usize_unsuffixed(index);
        let len = Literal::usize_unsuffixed(fields.unnamed.len());
        let unnamed = (0..fields.unnamed.len())
            .map(|index| Ident::new(&format!("feald_{}", index), Span::call_site()))
            .collect::<Vec<_>>();
        let pat = quote! {
            #ident::#variant_ident(#(#unnamed),*)
        };
        let serialize = quote! {
            let mut serializer = #_Serializer::serialize_tuple_variant(serializer, #ident_str, #index, #variant_ident_str, #len)?;
        };
        let fields = unnamed
            .iter()
            .map(|ident| {
                quote! {
                    #_SerializeTupleVariant::serialize_field(&mut serializer, &#ident)?;
                }
            })
            .collect();
        let end = quote! {
            #_SerializeTupleVariant::end(serializer)
        };
        (pat, serialize, fields, end)
    }
}
