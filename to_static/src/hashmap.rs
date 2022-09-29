use crate::ToStatic;
use std::{collections::HashMap, hash::Hash};

impl<K, V> ToStatic for HashMap<K, V>
where
    K: ToStatic,
    K::Static: Hash + Eq,
    V: ToStatic,
{
    type Static = HashMap<K::Static, V::Static>;
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
