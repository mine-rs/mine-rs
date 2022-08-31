use crate::ToStatic;

impl ToStatic for bool {
    type Static = bool;
    fn to_static(&self) -> Self::Static {
        *self
    }
    fn into_static(self) -> Self::Static {
        self
    }
}
