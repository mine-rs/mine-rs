#![doc(hidden)]

use crate::ToStatic;

mod bool;
mod buffer;
mod num;
mod string;
mod vec;

impl<T: ToStatic> ToStatic for Option<T> {
    type Static = Option<<T as ToStatic>::Static>;
    fn to_static(&self) -> Self::Static {
        self.as_ref().map(ToStatic::to_static)
    }
    fn into_static(self) -> Self::Static {
        self.map(ToStatic::into_static)
    }
}
