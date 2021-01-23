use proc_macro::TokenStream;

#[proc_macro_derive(VariantIdent)]
pub fn with_types(items: TokenStream) -> TokenStream {
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
