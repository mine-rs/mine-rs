use crate::ToStatic;
use std::collections::BTreeMap;

impl<K, V> ToStatic for BTreeMap<K, V>
where
    K: ToStatic,
    K::Static: Ord,
    V: ToStatic,
{
    type Static = BTreeMap<K::Static, V::Static>;
    fn to_static(&self) -> Self::Static {
        self.iter()
            .map(|(k, v)| (k.to_static(), v.to_static()))
            .collect()
    }
    fn into_static(self) -> Self::Static {
        self.into_iter()
            .map(|(k, v)| (k.into_static(), v.into_static()))
            .collect()
    }
}
