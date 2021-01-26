use proc_macro::TokenStream;

#[proc_macro_derive(VariantIdent, attributes(from))]
pub fn derive_variant_ident(items: TokenStream) -> TokenStream {
    sulafat_macros_impl::derive_variant_ident(items.into()).into()
}

#[proc_macro]
pub fn dbg(items: TokenStream) -> TokenStream {
    sulafat_macros_impl::dbg(items.into()).into()
}

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(items: TokenStream) -> TokenStream {
    sulafat_macros_impl::derive_serialize(items.into()).into()
}

#[proc_macro_derive(StyleSet, attributes(style_set))]
pub fn derive_style_set(items: TokenStream) -> TokenStream {
    sulafat_macros_impl::derive_style_set(items.into()).into()
}

#[proc_macro_derive(PartialEq)]
pub fn derive_partial_eq(items: TokenStream) -> TokenStream {
    sulafat_macros_impl::derive_partial_eq(items.into()).into()
}

#[proc_macro_derive(Clone)]
pub fn derive_clone(items: TokenStream) -> TokenStream {
    sulafat_macros_impl::derive_clone(items.into()).into()
}
