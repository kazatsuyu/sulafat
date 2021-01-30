use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::cell::Cell;
use sulafat_style::{Length, LengthOrPercentage, Parcentage, StyleRule, WritingMode};
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

struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    fn as_ref(&self) -> Wrapper<&T> {
        Wrapper(&self.0)
    }
}

impl ToTokens for Wrapper<Length> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ref().to_tokens(tokens)
    }
}

impl ToTokens for Wrapper<&Length> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(match &self.0 {
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

impl ToTokens for Wrapper<Parcentage> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ref().to_tokens(tokens)
    }
}

impl ToTokens for Wrapper<&Parcentage> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = self.0 .0;
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(quote! {::#sulafat_style::Parcentage(#value)})
    }
}

impl Parse for Wrapper<LengthOrPercentage> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<Lit>()?;
        Ok(Self(match lit {
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
        }?))
    }
}

impl ToTokens for Wrapper<LengthOrPercentage> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ref().to_tokens(tokens)
    }
}

impl ToTokens for Wrapper<&LengthOrPercentage> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(match &self.0 {
            LengthOrPercentage::Length(length) => {
                let length = Wrapper(length);
                quote! { ::#sulafat_style::LengthOrPercentage::Length(#length) }
            }
            LengthOrPercentage::Parcentage(parcentage) => {
                let parcentage = Wrapper(parcentage);
                quote! { ::#sulafat_style::LengthOrPercentage::Parcentage(#parcentage) }
            }
        })
    }
}

impl ToTokens for Wrapper<WritingMode> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ref().to_tokens(tokens)
    }
}

impl ToTokens for Wrapper<&WritingMode> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(match &self.0 {
            WritingMode::HorizontalTb => {
                quote! { ::#sulafat_style::WritingMode::HorizontalTb }
            }
            WritingMode::VerticalRl => {
                quote! { ::#sulafat_style::WritingMode::VerticalRl }
            }
            WritingMode::VerticalLr => {
                quote! { ::#sulafat_style::WritingMode::VerticalLr }
            }
            WritingMode::SidewayzRl => {
                quote! { ::#sulafat_style::WritingMode::SidewayzRl }
            }
            WritingMode::SidewayzLr => {
                quote! { ::#sulafat_style::WritingMode::SidewayzLr }
            }
        })
    }
}

impl Parse for Wrapper<WritingMode> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident1 = input.parse::<Ident>()?;
        input.parse::<Token![-]>()?;
        let ident2 = input.parse::<Ident>()?;
        let err = || {
            Err(syn::Error::new(
                Span::call_site(),
                &format!("Unexpected value {}-{}", ident1, ident2),
            ))
        };
        Ok(Wrapper(if ident1 == "horizontal" && ident2 == "Tb" {
            WritingMode::HorizontalTb
        } else if ident1 == "vertical" {
            if ident2 == "rl" {
                WritingMode::VerticalRl
            } else if ident2 == "lr" {
                WritingMode::VerticalLr
            } else {
                return err();
            }
        } else if ident1 == "sideways" {
            if ident2 == "rl" {
                WritingMode::SidewayzRl
            } else if ident2 == "lr" {
                WritingMode::SidewayzLr
            } else {
                return err();
            }
        } else {
            return err();
        }))
    }
}

impl Parse for Wrapper<StyleRule> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let rule = if name == "left" {
            input.parse::<Token![:]>()?;
            StyleRule::Left(input.parse::<Wrapper<_>>()?.0)
        } else if name == "right" {
            input.parse::<Token![:]>()?;
            StyleRule::Right(input.parse::<Wrapper<_>>()?.0)
        } else if name == "writing" {
            input.parse::<Token![-]>()?;
            let name2 = input.parse::<Ident>()?;
            if name2 == "mode" {
                input.parse::<Token![:]>()?;
                StyleRule::WritingMode(input.parse::<Wrapper<_>>()?.0)
            } else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    &format!("Unexpected rule name {}-{}", name, name2),
                ));
            }
        } else {
            return Err(syn::Error::new(
                Span::call_site(),
                &format!("Unexpected rule name {}", name),
            ));
        };
        input.parse::<Token![;]>()?;
        Ok(Self(rule))
    }
}

impl ToTokens for Wrapper<StyleRule> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ref().to_tokens(tokens)
    }
}

impl ToTokens for Wrapper<&StyleRule> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sulafat_style = crate_name("sulafat-style");
        tokens.extend(match &self.0 {
            StyleRule::Left(left) => {
                let left = Wrapper(left);
                quote! { ::#sulafat_style::StyleRule::Left(#left) }
            }
            StyleRule::Right(right) => {
                let right = Wrapper(right);
                quote! { ::#sulafat_style::StyleRule::Right(#right) }
            }
            StyleRule::WritingMode(writing_mode) => {
                let writing_mode = Wrapper(writing_mode);
                quote! { ::#sulafat_style::StyleRule::WritingMode(#writing_mode) }
            }
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
            rules.push(input.parse::<Wrapper<StyleRule>>()?.0);
        }
        Ok(Self { name, rules })
    }
}

impl ToTokens for StyleRules {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for rule in &self.rules {
            let rule = Wrapper(rule);
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
                    write!(writer, "{}", rule).unwrap();
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
