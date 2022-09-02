mod bool;
mod cow;
mod num;
mod option;
mod string;
mod uuid;
mod vec;

mod encoding;

pub trait ToStatic {
    type Static: 'static;
    fn to_static(&self) -> Self::Static;
    fn into_static(self) -> Self::Static;
}
