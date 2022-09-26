use crate::ToStatic;

impl ToStatic for uuid::Uuid {
    type Static = uuid::Uuid;
    fn to_static(&self) -> Self::Static {
        *self
    }
    fn into_static(self) -> Self::Static {
        self
    }
}
