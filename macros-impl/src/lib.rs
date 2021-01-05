use heck::SnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2, Fields, GenericParam, Ident, ItemEnum};

fn with_types_impl(args: TokenStream, items: TokenStream) -> syn::Result<TokenStream> {
    let item_enum = parse2::<ItemEnum>(items.clone())?;
    let vis = &item_enum.vis;
    let types_ident = parse2::<Option<Ident>>(args)?
        .unwrap_or_else(|| Ident::new(&format!("{}Type", item_enum.ident), Span::call_site()));
    let variants = item_enum.variants.iter().map(|variant| &variant.ident);
    let ident = &item_enum.ident;
    let snake_types_ident = Ident::new(&types_ident.to_string().to_snake_case(), Span::call_site());
    let match_arms = item_enum.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        match &variant.fields {
            Fields::Unit => quote! { #ident::#variant_ident => #types_ident::#variant_ident, },
            Fields::Unnamed(..) => {
                quote! { #ident::#variant_ident(..) => #types_ident::#variant_ident, }
            }
            Fields::Named(..) => {
                quote! { #ident::#variant_ident{..} => #types_ident::#variant_ident, }
            }
        }
    });
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let params = generics.params.iter().map(|param| match param {
        GenericParam::Lifetime(lifetime) => {
            let lifetime = &lifetime.lifetime;
            quote!(#lifetime)
        }
        GenericParam::Type(param) => {
            let ident = &param.ident;
            quote!(#ident)
        }
        GenericParam::Const(param) => {
            let ident = &param.ident;
            quote!(#ident)
        }
    });
    let lt_token = generics.lt_token;
    let gt_token = generics.gt_token;
    Ok(quote! {
        #items
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
        #vis enum #types_ident {
            #(#variants,)*
        }
        impl #generics #ident #lt_token #(#params),* #gt_token #where_clause {
            #vis fn #snake_types_ident(&self) -> #types_ident {
                match self {
                    #(#match_arms)*
                }
            }
        }
    })
}

pub fn with_types(args: TokenStream, items: TokenStream) -> TokenStream {
    with_types_impl(args, items).unwrap_or_else(|e| e.to_compile_error())
}
