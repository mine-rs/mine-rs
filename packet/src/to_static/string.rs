use crate::ToStatic;

impl ToStatic for String {
    type Static = String;
    fn to_static(&self) -> Self::Static {
        self.clone()
    }
    fn into_static(self) -> Self::Static {
        self
    }
}
