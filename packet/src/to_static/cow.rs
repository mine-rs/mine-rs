use std::borrow::Cow;

use crate::ToStatic;

impl<T: ?Sized> ToStatic for Cow<'_, T>
where
    T: ToOwned + 'static,
{
    type Static = Cow<'static, T>;
    fn to_static(&self) -> Self::Static {
        Self::Static::Owned(self.clone().into_owned())
    }
    fn into_static(self) -> Self::Static {
        Self::Static::Owned(self.into_owned())
    }
}
