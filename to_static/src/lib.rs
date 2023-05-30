mod bool;
mod btreemap;
mod cow;
mod hashmap;
mod nonzero;
mod num;
mod option;
mod string;
#[cfg(feature = "uuid")]
mod uuid;
mod vec;

pub trait ToStatic {
    type Static: 'static;
    fn to_static(&self) -> Self::Static;
    fn into_static(self) -> Self::Static;
}
