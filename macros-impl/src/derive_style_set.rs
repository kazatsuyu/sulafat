use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::cell::Cell;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse2, Attribute, ItemStruct, Lit, Token,
};

use crate::util::crate_name;

#[cfg(feature = "export-css")]
use {
    crate::util::out_dir,
    std::{
        cell::RefCell,
        fs::File,
        io::{BufWriter, Write},
        path::Path,
    },
};

enum Length {
    Em(f64),
    Px(f64),
    Vh(f64),
    Vw(f64),
}

impl ToTokens for Length {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(match self {
            Length::Em(em) => {
                quote! { ::#sulafat_style::Length::Em(#em) }
            }
            Length::Px(px) => {
                quote! { ::#sulafat_style::Length::Px(#px) }
            }
            Length::Vh(em) => {
                quote! { ::#sulafat_style::Length::Vh(#em) }
            }
            Length::Vw(px) => {
                quote! { ::#sulafat_style::Length::Vw(#px) }
            }
        })
    }
}

struct Parcentage(f64);

impl ToTokens for Parcentage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0;
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(quote! {::#sulafat_style::Parcentage(#value)})
    }
}

enum LengthOrPercentage {
    Length(Length),
    Parcentage(Parcentage),
}

impl Parse for LengthOrPercentage {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<Lit>()?;
        match lit {
            Lit::Int(lit) => match lit.suffix() {
                "em" => Ok(LengthOrPercentage::Length(Length::Em(lit.base10_parse()?))),
                "px" => Ok(LengthOrPercentage::Length(Length::Px(lit.base10_parse()?))),
                "vh" => Ok(LengthOrPercentage::Length(Length::Vh(lit.base10_parse()?))),
                "vw" => Ok(LengthOrPercentage::Length(Length::Vw(lit.base10_parse()?))),
                "" => {
                    if input.peek(Token![%]) {
                        input.parse::<Token![%]>()?;
                        Ok(LengthOrPercentage::Parcentage(Parcentage(
                            lit.base10_parse()?,
                        )))
                    } else {
                        if let Ok(0) = lit.base10_parse() {
                            Ok(LengthOrPercentage::Length(Length::Px(0.)))
                        } else {
                            Err(syn::Error::new(lit.span(), "Suffix is required."))
                        }
                    }
                }
                _ => Err(syn::Error::new(
                    lit.span(),
                    &format!("Unexpected suffix {}", lit.suffix()),
                )),
            },
            Lit::Float(lit) => match lit.suffix() {
                "em" => Ok(LengthOrPercentage::Length(Length::Em(lit.base10_parse()?))),
                "px" => Ok(LengthOrPercentage::Length(Length::Px(lit.base10_parse()?))),
                "vh" => Ok(LengthOrPercentage::Length(Length::Vh(lit.base10_parse()?))),
                "vw" => Ok(LengthOrPercentage::Length(Length::Vw(lit.base10_parse()?))),
                "" => {
                    if input.peek(Token![%]) {
                        input.parse::<Token![%]>()?;
                        Ok(LengthOrPercentage::Parcentage(Parcentage(
                            lit.base10_parse()?,
                        )))
                    } else {
                        if let Ok(0) = lit.base10_parse() {
                            Ok(LengthOrPercentage::Length(Length::Px(0.)))
                        } else {
                            Err(syn::Error::new(lit.span(), "Suffix is required."))
                        }
                    }
                }
                _ => Err(syn::Error::new(
                    lit.span(),
                    &format!("Unexpected suffix {}", lit.suffix()),
                )),
            },
            _ => Err(syn::Error::new(
                lit.span(),
                &format!("Unexpected value {}", &lit.into_token_stream().to_string()),
            )),
        }
    }
}

impl ToTokens for LengthOrPercentage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(match self {
            LengthOrPercentage::Length(length) => {
                quote! { ::#sulafat_style::LengthOrPercentage::Length(#length) }
            }
            LengthOrPercentage::Parcentage(parcentage) => {
                quote! { ::#sulafat_style::LengthOrPercentage::Parcentage(#parcentage) }
            }
        })
    }
}

enum StyleRule {
    Left(LengthOrPercentage),
    Right(LengthOrPercentage),
}

impl Parse for StyleRule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let rule = if name == "left" {
            StyleRule::Left(input.parse()?)
        } else if name == "right" {
            StyleRule::Right(input.parse()?)
        } else {
            return Err(syn::Error::new(
                Span::call_site(),
                &format!("Unexpected rule name {}", name),
            ));
        };
        input.parse::<Token![;]>()?;
        Ok(rule)
    }
}

impl ToTokens for StyleRule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(match self {
            StyleRule::Left(left) => quote! { ::#sulafat_style::StyleRule::Left(#left) },
            StyleRule::Right(right) => quote! { ::#sulafat_style::StyleRule::Right(#right) },
        })
    }
}

pub struct StyleRules {
    name: String,
    rules: Vec<StyleRule>,
}

fn generate_name() -> String {
    thread_local! {
        static I: Cell<u64> = Cell::new(0);
    }
    format!(
        "sulafat-{}",
        I.with(|i| {
            let j = i.get();
            i.set(j + 1);
            j
        })
    )
}

impl Parse for StyleRules {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let buffer;
        let (name, input) = if input.parse::<Token![.]>().is_ok() {
            let ident = input.parse::<Ident>()?;
            braced!(buffer in input);
            (ident.to_string(), &buffer)
        } else {
            (generate_name(), input)
        };
        let mut rules = vec![];
        while !input.is_empty() {
            rules.push(input.parse()?);
        }
        Ok(Self { name, rules })
    }
}

impl ToTokens for StyleRules {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for rule in &self.rules {
            tokens.extend(quote! {
                #rule,
            })
        }
    }
}

fn parse_attrs(attrs: &[Attribute]) -> syn::Result<StyleRules> {
    for attr in attrs {
        let path = &attr.path;
        if path.leading_colon.is_none()
            && path.segments.len() == 1
            && path.segments[0].ident == "style_set"
        {
            return attr.parse_args();
        }
    }
    Ok(StyleRules {
        name: generate_name(),
        rules: vec![],
    })
}

#[cfg(feature = "export-css")]
fn file<F: FnOnce(&mut BufWriter<File>)>(path: &Path, f: F) {
    thread_local! {
        static FILE : RefCell<Option<BufWriter<File>>> = RefCell::new(None);
    }
    FILE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        f(borrow.get_or_insert_with(|| {
            let file = File::create(path).unwrap();
            BufWriter::new(file)
        }))
    })
}

fn derive_style_set_impl(items: TokenStream) -> syn::Result<TokenStream> {
    let item = parse2::<ItemStruct>(items)?;
    let ident = &item.ident;
    let rules = parse_attrs(&item.attrs)?;
    let name = &rules.name;
    #[cfg(feature = "export-css")]
    if !rules.rules.is_empty() {
        if let Some(path) = out_dir() {
            let path = Path::new(&path).join("style.css");
            file(&path, |writer| {
                write!(writer, ".{}{{", name).unwrap();
                for rule in &rules.rules {
                    match rule {
                        StyleRule::Left(LengthOrPercentage::Length(Length::Em(em))) => {
                            write!(writer, "left:{}em;", em)
                        }
                        StyleRule::Left(LengthOrPercentage::Length(Length::Px(px))) => {
                            write!(writer, "left:{}px;", px)
                        }
                        StyleRule::Left(LengthOrPercentage::Length(Length::Vh(vh))) => {
                            write!(writer, "left:{}vh;", vh)
                        }
                        StyleRule::Left(LengthOrPercentage::Length(Length::Vw(vw))) => {
                            write!(writer, "left:{}vw;", vw)
                        }
                        StyleRule::Left(LengthOrPercentage::Parcentage(Parcentage(parcentage))) => {
                            write!(writer, "left:{}%;", parcentage)
                        }
                        StyleRule::Right(LengthOrPercentage::Length(Length::Em(em))) => {
                            write!(writer, "right:{}em;", em)
                        }
                        StyleRule::Right(LengthOrPercentage::Length(Length::Px(px))) => {
                            write!(writer, "right:{}px;", px)
                        }
                        StyleRule::Right(LengthOrPercentage::Length(Length::Vh(vh))) => {
                            write!(writer, "right:{}vh;", vh)
                        }
                        StyleRule::Right(LengthOrPercentage::Length(Length::Vw(vw))) => {
                            write!(writer, "right:{}vw;", vw)
                        }
                        StyleRule::Right(LengthOrPercentage::Parcentage(Parcentage(
                            parcentage,
                        ))) => {
                            write!(writer, "right:{}%;", parcentage)
                        }
                    }
                    .unwrap()
                }
                write!(writer, "}}").unwrap();
            })
        }
    }
    let sulafat_style = crate_name("sulafat-style");
    Ok(quote! {
        impl ::#sulafat_style::StyleSet for #ident {
            fn name() -> String {
                #name.to_string()
            }
            fn rules() -> &'static [::#sulafat_style::StyleRule] {
                &[
                    #rules
                ]
            }
        }
        const _: () = {
            thread_local! {
                static A: () = {
                    ::#sulafat_style::export::<#ident>();
                };
            }
        };
    })
}

pub fn derive_style_set(items: TokenStream) -> TokenStream {
    derive_style_set_impl(items).unwrap_or_else(|e| e.into_compile_error())
}
