use std::fmt::{self, Formatter};

use crate::{PatchElement, Single};
use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_derive::Deserialize;
use sulafat_macros::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum PatchSingle<Msg> {
    Replace(Single<Msg>),
    Element(PatchElement<Msg>),
}
struct PatchSingleVisitor<Msg>(std::marker::PhantomData<Msg>);

impl<'de, Msg> Visitor<'de> for PatchSingleVisitor<Msg> {
    type Value = PatchSingle<Msg>;
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "variant of Replace or Element")
    }
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        #[derive(Deserialize)]
        enum VariantTag {
            Replace,
            Element,
        }
        let (v, variant) = data.variant::<VariantTag>()?;
        Ok(match v {
            VariantTag::Replace => PatchSingle::Replace(variant.newtype_variant()?),
            VariantTag::Element => PatchSingle::Element(variant.newtype_variant()?),
        })
    }
}

impl<'de, Msg> Deserialize<'de> for PatchSingle<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchSingleVisitor<Msg>(std::marker::PhantomData<Msg>);

        impl<'de, Msg> Visitor<'de> for PatchSingleVisitor<Msg> {
            type Value = PatchSingle<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant of Replace or Element")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Deserialize)]
                enum VariantTag {
                    Replace,
                    Element,
                }
                let (v, variant) = data.variant::<VariantTag>()?;
                Ok(match v {
                    VariantTag::Replace => PatchSingle::Replace(variant.newtype_variant()?),
                    VariantTag::Element => PatchSingle::Element(variant.newtype_variant()?),
                })
            }
        }
        deserializer.deserialize_enum(
            "PatchSingle",
            &["Replace", "Element"],
            PatchSingleVisitor(Default::default()),
        )
    }
}
