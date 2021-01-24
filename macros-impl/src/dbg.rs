use proc_macro2::TokenStream;
use quote::quote;

fn dbg_impl(items: TokenStream) -> syn::Result<TokenStream> {
    Ok(quote! {
        match #items {
            #[cfg(target_arch = "wasm32")]
            tmp => {
                crate::util::debug(&format!("[{}:{}] {:?}", file!(), line!(), tmp).into());
                tmp
            }
            #[cfg(not(target_arch = "wasm32"))]
            tmp => {
                ::std::dbg!(tmp)
            }
        }
    })
}

pub fn dbg(items: TokenStream) -> TokenStream {
    dbg_impl(items).unwrap_or_else(|e| e.to_compile_error())
}
