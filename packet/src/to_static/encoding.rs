use miners_encoding::attrs::Angle;

use crate::ToStatic;

impl ToStatic for Angle {
    type Static = Angle;
    fn to_static(&self) -> Self::Static {
        Self::Static::from(*self.as_ref())
    }
    fn into_static(self) -> Self::Static {
        self
    }
}
