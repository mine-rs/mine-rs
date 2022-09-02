use crate::ToStatic;

impl<T: ToStatic> ToStatic for Vec<T> {
    type Static = Vec<<T as ToStatic>::Static>;
    fn to_static(&self) -> Self::Static {
        self.iter().map(ToStatic::to_static).collect()
    }
    fn into_static(self) -> Self::Static {
        self.into_iter().map(ToStatic::into_static).collect()
    }
}
