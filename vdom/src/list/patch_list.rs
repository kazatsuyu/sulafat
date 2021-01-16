use std::fmt::{self, Formatter};

use crate::{PatchSingle, Single};
use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::Deserialize;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PatchList<Msg> {
    All(Vec<PatchListOp<Msg>>),
    Entries(usize, Vec<(usize, PatchSingle<Msg>)>),
    Truncate(usize),
}

impl<Msg> Serialize for PatchList<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PatchList::All(all) => {
                let mut variant = serializer.serialize_tuple_variant("PatchList", 0, "All", 1)?;
                variant.serialize_field(all)?;
                variant.end()
            }
            PatchList::Entries(len, entries) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchList", 1, "Entries", 2)?;
                variant.serialize_field(len)?;
                variant.serialize_field(entries)?;
                variant.end()
            }
            PatchList::Truncate(len) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchList", 2, "Truncate", 1)?;
                variant.serialize_field(len)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for PatchList<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchListVisitor<Msg>(std::marker::PhantomData<Msg>);

        impl<'de, Msg> Visitor<'de> for PatchListVisitor<Msg> {
            type Value = PatchList<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant of All, Entries, or Truncate")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Deserialize)]
                enum VariantTag {
                    All,
                    Entries,
                    Truncate,
                }
                let (v, variant) = data.variant::<VariantTag>()?;
                Ok(match v {
                    VariantTag::All => PatchList::All(variant.newtype_variant()?),
                    VariantTag::Entries => {
                        let (len, entries) = variant.newtype_variant()?;
                        PatchList::Entries(len, entries)
                    }
                    VariantTag::Truncate => PatchList::Truncate(variant.newtype_variant()?),
                })
            }
        }
        deserializer.deserialize_enum(
            "PatchList",
            &["All", "Modify", "From", "FromModify", "New"],
            PatchListVisitor(Default::default()),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PatchListOp<Msg> {
    Nop,
    Modify(PatchSingle<Msg>),
    From(usize),
    FromModify(usize, PatchSingle<Msg>),
    New(Single<Msg>),
}

impl<Msg> Serialize for PatchListOp<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PatchListOp::Nop => serializer.serialize_unit_variant("PatchListOp", 0, "Nop"),
            PatchListOp::Modify(patch) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchListOp", 1, "Modify", 1)?;
                variant.serialize_field(patch)?;
                variant.end()
            }
            PatchListOp::From(len) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchListOp", 2, "From", 1)?;
                variant.serialize_field(len)?;
                variant.end()
            }
            PatchListOp::FromModify(len, patch) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchListOp", 3, "FromModify", 2)?;
                variant.serialize_field(len)?;
                variant.serialize_field(patch)?;
                variant.end()
            }
            PatchListOp::New(single) => {
                let mut variant = serializer.serialize_tuple_variant("PatchListOp", 4, "New", 1)?;
                variant.serialize_field(single)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for PatchListOp<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchListOpVisitor<Msg>(std::marker::PhantomData<Msg>);

        impl<'de, Msg> Visitor<'de> for PatchListOpVisitor<Msg> {
            type Value = PatchListOp<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "variant of Nop, Modify, From, FromModify, or New"
                )
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Deserialize)]
                enum VariantTag {
                    Nop,
                    Modify,
                    From,
                    FromModify,
                    New,
                }
                let (v, variant) = data.variant::<VariantTag>()?;
                Ok(match v {
                    VariantTag::Nop => PatchListOp::Nop,
                    VariantTag::Modify => PatchListOp::Modify(variant.newtype_variant()?),
                    VariantTag::From => PatchListOp::From(variant.newtype_variant()?),
                    VariantTag::FromModify => {
                        let (index, patch) = variant.newtype_variant()?;
                        PatchListOp::FromModify(index, patch)
                    }
                    VariantTag::New => PatchListOp::New(variant.newtype_variant()?),
                })
            }
        }
        deserializer.deserialize_enum(
            "PatchListOp",
            &["Nop", "Modify", "From", "FromModify", "New"],
            PatchListOpVisitor(Default::default()),
        )
    }
}
