mod bool;
mod btreemap;
mod cow;
mod num;
mod option;
mod string;
#[cfg(feature = "uuid")]
mod uuid;
mod vec;
mod hashmap;

pub trait ToStatic {
    type Static: 'static;
    fn to_static(&self) -> Self::Static;
    fn into_static(self) -> Self::Static;
}
