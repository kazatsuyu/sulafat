use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn with_types(args: TokenStream, items: TokenStream) -> TokenStream {
    sulafat_macros_impl::with_types(args.into(), items.into()).into()
}
