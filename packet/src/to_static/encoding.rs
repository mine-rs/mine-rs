use miners_encoding::attrs::Angle;
use miners_encoding::attrs::StringUuid;

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
impl ToStatic for StringUuid {
    type Static = StringUuid;
    fn to_static(&self) -> Self::Static {
        self.clone()
    }
    fn into_static(self) -> Self::Static {
        self
    }
}
