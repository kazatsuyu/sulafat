use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

macro_rules! __fq_name {
    ($([$ident:ident, $fq_name:ty]),* $(,)?) => {$(
        #[allow(non_camel_case_types)]
        pub(crate) struct $ident;
        impl ToTokens for $ident {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                tokens.extend(quote! { $fq_name });
            }
        }
    )*};
}

__fq_name! {
    [_Result, ::std::result::Result],
    [_Serialize, ::serde::Serialize],
    [_Serializer, ::serde::Serializer],
    [_SerializeStruct, ::serde::ser::SerializeStruct],
    [_SerializeTupleStruct, ::serde::ser::SerializeTupleStruct],
    [_SerializeStructVariant, ::serde::ser::SerializeStructVariant],
    [_SerializeTupleVariant, ::serde::ser::SerializeTupleVariant],
}
