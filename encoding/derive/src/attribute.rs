use darling::FromMeta;
use proc_macro2::Ident;

#[derive(Clone)]
pub struct Fixed {
    pub precision: u8,
    pub typ: Ident,
}
impl FromMeta for Fixed {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        (match *item {
            syn::Meta::Path(_) => Self::from_word(),
            syn::Meta::List(ref value) => {
                let mut values = value.nested.iter();
                let Some(first) = values.next() else {
                    return Err(darling::Error::custom(
                        "supplied no arguments to fixed, required: (precision, type)",
                    )
                    .with_span(&item));
                };
                let Some(second) = values.next() else {
                    return Err(darling::Error::custom(
                        "supplied only one argument to fixed, required: (precision, type)",
                    )
                    .with_span(&item));
                };
                if values.next().is_some() {
                    return Err(darling::Error::custom(
                        "supplied more than 2 arguments (precision, type) to fixed",
                    )
                    .with_span(&item));
                };
                Ok(Fixed {
                    precision: u8::from_nested_meta(first)?,
                    typ: Ident::from_nested_meta(second)?,
                })
            }
            syn::Meta::NameValue(ref value) => Self::from_value(&value.lit),
        })
        .map_err(|e| e.with_span(item))
    }
}
