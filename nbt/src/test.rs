use std::{borrow::Cow, collections::HashMap};

use crate::de::{self, CompoundAccess, Deserialize, Deserializer};

// #[derive(Nbt)]
struct NbtEncodable {
    // #[nbt()]
    thing: Vec<i8>,
    other: i32,
    idk: u8,
}

// impl<'de, K, V> de::Visitor<'de> for HashMap<K, V>
// where
//     for<'a> K: From<Cow<'a, str>> + core::hash::Hash + core::cmp::Eq + std::fmt::Debug,
//     V: for<'a> Deserialize<'a>,
// {
//     type Value = Self;

//     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         formatter.write_str("compound")
//     }

//     fn visit_compound<A>(
//         self,
//         mut compound: A,
//     ) -> Result<Self::Value, A::Error>
//     where
//         A: CompoundAccess<'de>,
//     {
//         let mut this = match compound.size_hint() {
//             Some(hint) => HashMap::with_capacity(hint),
//             None => HashMap::new(),
//         };
//         while let Some((key, val)) = compound.next_entry()? {
//             use std::collections::hash_map::Entry::*;
//             match this.entry(<K as core::convert::From<Cow<str>>>::from(key)) {
//                 Occupied(o) => {
//                     return Err(de::Error::custom(format!("duplicate field {:?}", o.key())))
//                 }
//                 Vacant(v) => {
//                     v.insert(val);
//                 }
//             }
//         }
//         Ok(this)
//     }
// }

// impl<'de, K, V> Deserialize<'de> for HashMap<K, V>
// where
//     for<'a> K: From<Cow<'a, str>> + core::hash::Hash + core::cmp::Eq + std::fmt::Debug,
//     V: for<'a> Deserialize<'a>,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct Visitor<K, V> {
//             k: core::marker::PhantomData<K>,
//             v: core::marker::PhantomData<V>,
//         }

//         impl<'de, K, V> de::Visitor<'de> for Visitor<K, V>
//         where
//             for<'a> K: From<Cow<'a, str>> + core::hash::Hash + core::cmp::Eq + std::fmt::Debug,
//             V: for<'a> Deserialize<'a>,
//         {
//             type Value = HashMap<K, V>;

//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("compound")
//             }

//             fn visit_compound<A>(
//                 self,
//                 mut compound: A,
//             ) -> Result<Self::Value, A::Error>
//             where
//                 A: CompoundAccess<'de>,
//             {
//                 let mut this = match compound.size_hint() {
//                     Some(hint) => HashMap::with_capacity(hint),
//                     None => HashMap::new(),
//                 };
//                 while let Some((key, val)) = compound.next_entry()? {
//                     use std::collections::hash_map::Entry::*;
//                     match this.entry(<K as core::convert::From<Cow<str>>>::from(key)) {
//                         Occupied(o) => {
//                             return Err(de::Error::custom(format!("duplicate field {:?}", o.key())))
//                         }
//                         Vacant(v) => {
//                             v.insert(val);
//                         }
//                     }
//                 }
//                 Ok(this)
//             }
//         }

//         deserializer.deserialize(Visitor {
//             k: core::marker::PhantomData,
//             v: core::marker::PhantomData,
//         })
//     }
// }

impl<'de> Deserialize<'de> for NbtEncodable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = NbtEncodable;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("compound")
            }

            fn visit_compound<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: CompoundAccess<'de>,
            {
                let mut thing = None;
                let mut other = None;
                let mut idk = None;
                while let Some((name, deser)) = map.next_key()? {
                    match &name[..] {
                        "thing" => match thing {
                            Some(_) => return Err(de::Error::duplicate_field("thing")),
                            None => thing = Some(deser.value()?),
                        },
                        "other" => match other {
                            Some(_) => return Err(de::Error::duplicate_field("other")),
                            None => other = Some(deser.value()?),
                        },
                        "idk" => match idk {
                            Some(_) => return Err(de::Error::duplicate_field("other")),
                            None => idk = Some(deser.value()?),
                        },
                        _ => {}
                    }
                }
                let Some(thing) = thing else {return Err(de::Error::missing_field("thing"))};
                let Some(other) = other else {return Err(de::Error::missing_field("other"))};
                let Some(idk) = idk else {return Err(de::Error::missing_field("idk"))};
                Ok(NbtEncodable { thing, other, idk })
            }
        }

        deserializer.deserialize(Visitor)
    }
}
