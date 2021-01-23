pub(crate) mod fq_name;

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::collections::HashSet;
use syn::{punctuated::Punctuated, GenericParam, Generics, Lifetime, Token};

#[cfg(feature = "export-css")]
use std::env::{args, var};

pub(crate) enum Param<'a> {
    LifeTime(&'a Lifetime),
    Ident(&'a Ident),
}

impl<'a> ToTokens for Param<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Param::LifeTime(lifetime) => lifetime.to_tokens(tokens),
            Param::Ident(ident) => ident.to_tokens(tokens),
        }
    }
}

impl<'a> ToString for Param<'a> {
    fn to_string(&self) -> String {
        match self {
            Param::LifeTime(lifetime) => lifetime.to_string(),
            Param::Ident(ident) => ident.to_string(),
        }
    }
}

pub(crate) struct Params<'a> {
    pub(crate) lt_token: &'a Option<Token![<]>,
    pub(crate) gt_token: &'a Option<Token![>]>,
    pub(crate) params: Punctuated<Param<'a>, Token![,]>,
}

impl<'a> From<&'a Generics> for Params<'a> {
    fn from(generics: &'a Generics) -> Self {
        let params = generics
            .params
            .iter()
            .map(|param| match param {
                GenericParam::Lifetime(param) => Param::LifeTime(&param.lifetime),
                GenericParam::Type(param) => Param::Ident(&param.ident),
                GenericParam::Const(param) => Param::Ident(&param.ident),
            })
            .collect();
        Self {
            lt_token: &generics.lt_token,
            gt_token: &generics.gt_token,
            params,
        }
    }
}

impl<'a> ToTokens for Params<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lt_token.to_tokens(tokens);
        self.params.to_tokens(tokens);
        self.gt_token.to_tokens(tokens);
    }
}

pub(crate) fn new_name(names: &mut HashSet<String>, base_name: &str, index: &mut usize) -> String {
    if !names.contains(base_name) {
        let new_name = base_name.to_string();
        names.insert(new_name.clone());
        return new_name;
    }
    loop {
        let new_ident = format!("{}{}", base_name, index);
        *index += 1;
        if names.contains(&new_ident) {
            continue;
        }
        names.insert(new_ident.clone());
        break new_ident;
    }
}

pub(crate) fn new_ident(names: &mut HashSet<String>, base_name: &str, index: &mut usize) -> Ident {
    Ident::new(&new_name(names, base_name, index), Span::call_site())
}

#[cfg(feature = "export-css")]
pub(crate) fn out_dir() -> Option<String> {
    var("OUT_DIR").ok().or_else(|| {
        let mut args = args();
        while let Some(arg) = args.next() {
            if arg == "--out-dir" {
                return args.next();
            }
        }
        // panic!("Output directry not found either $OUT_DIR or --out-dir.");
        None
    })
}
