pub trait VariantIdent {
    type Type;
    fn variant_ident(&self) -> Self::Type;
}
