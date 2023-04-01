use crate::de::{Deserialize, Visitor};

macro_rules! deserialize_int {
    ($u:ident $i:ident $visit:ident) => {
        impl<'de> Deserialize<'de> for $u {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: super::Deserializer<'de>,
            {
                struct PrimitiveVisitor;

                impl<'de> Visitor<'de> for PrimitiveVisitor {
                    type Value = $u;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(stringify!($i))
                    }

                    fn $visit<E>(self, v: $i) -> Result<Self::Value, E>
                    where
                        E: crate::de::Error,
                    {
                        Ok(v as $u)
                    }
                }

                deserializer.deserialize(PrimitiveVisitor)
            }
        }
    };
}

deserialize_int!(i8 i8 visit_byte);
deserialize_int!(u8 i8 visit_byte);
deserialize_int!(i16 i16 visit_short);
deserialize_int!(u16 i16 visit_short);
deserialize_int!(i32 i32 visit_int);
deserialize_int!(u32 i32 visit_int);
deserialize_int!(i64 i64 visit_long);
deserialize_int!(u64 i64 visit_long);

impl<'de> Deserialize<'de> for Vec<i8> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: super::Deserializer<'de>,
    {
        struct VecVisitor;

        impl<'de> Visitor<'de> for VecVisitor {
            type Value = Vec<i8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(stringify!([i8]))
            }

            fn visit_bytearray<E>(self, v: std::borrow::Cow<'de, [u8]>) -> Result<Self::Value, E>
            where
                E: crate::de::Error,
            {
                Ok(match v {
                    std::borrow::Cow::Borrowed(b) => {
                        let slice = unsafe {
                            core::slice::from_raw_parts(b.as_ptr() as *const i8, b.len())
                        };
                        slice.to_vec()
                    }
                    std::borrow::Cow::Owned(vec) => {
                        let mut vec = core::mem::ManuallyDrop::new(vec);
                        unsafe {
                            Vec::from_raw_parts(
                                vec.as_mut_ptr() as *mut i8,
                                vec.len(),
                                vec.capacity(),
                            )
                        }
                    }
                })
            }

            fn visit_list_byte<E>(self, v: std::borrow::Cow<'de, [i8]>) -> Result<Self::Value, E>
            where
                E: crate::de::Error,
            {
                Ok(match v {
                    std::borrow::Cow::Borrowed(b) => b.to_vec(),
                    std::borrow::Cow::Owned(vec) => vec,
                })
            }
        }

        deserializer.deserialize(VecVisitor)
    }
}
impl<'de> Deserialize<'de> for Vec<u8> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: super::Deserializer<'de>,
    {
        struct VecVisitor;

        impl<'de> Visitor<'de> for VecVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(stringify!([i8]))
            }

            fn visit_bytearray<E>(self, v: std::borrow::Cow<'de, [u8]>) -> Result<Self::Value, E>
            where
                E: crate::de::Error,
            {
                Ok(match v {
                    std::borrow::Cow::Borrowed(b) => b.to_vec(),
                    std::borrow::Cow::Owned(vec) => vec,
                })
            }

            fn visit_list_byte<E>(self, v: std::borrow::Cow<'de, [i8]>) -> Result<Self::Value, E>
            where
                E: crate::de::Error,
            {
                Ok(match v {
                    std::borrow::Cow::Borrowed(b) => {
                        let slice = unsafe {
                            core::slice::from_raw_parts(b.as_ptr() as *const u8, b.len())
                        };
                        slice.to_vec()
                    }
                    std::borrow::Cow::Owned(vec) => {
                        let mut vec = core::mem::ManuallyDrop::new(vec);
                        unsafe {
                            Vec::from_raw_parts(
                                vec.as_mut_ptr() as *mut u8,
                                vec.len(),
                                vec.capacity(),
                            )
                        }
                    }
                })
            }
        }

        deserializer.deserialize(VecVisitor)
    }
}

struct List;
struct Array;

struct TagEncoder<T, Tag> {
    t: T,
    tag: std::marker::PhantomData<Tag>,
}

impl<'de> Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: super::Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_string<E>(self, v: std::borrow::Cow<'de, str>) -> Result<Self::Value, E>
            where
                E: crate::de::Error,
            {
                Ok(v.into_owned())
            }
        }

        deserializer.deserialize(StringVisitor)
    }
}

macro_rules! deserialize_other {
    ($id:ident $visit:ident $expecting:literal $ty:ty) => {
        pub struct $id<'de> (pub $ty);
        impl <'a> ::miners_to_static::ToStatic for $id<'a> {
            type Static = $id<'static>;
            fn to_static(&self) -> Self::Static {
                $id(::miners_to_static::ToStatic::to_static(&self.0))
            }
            fn into_static(self) -> Self::Static {
                $id(::miners_to_static::ToStatic::into_static(self.0))
            }
        }
        impl<'de> Deserialize<'de> for $id<'de> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: super::Deserializer<'de>,
            {
                struct ForwardVisitor;

                impl<'de> Visitor<'de> for ForwardVisitor {
                    type Value = $id<'de>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str($expecting)
                    }

                    fn $visit<E>(self, v: $ty) -> Result<Self::Value, E>
                        where
                            E: crate::de::Error, {
                        Ok($id(v))
                    }
                }

                deserializer.deserialize(ForwardVisitor)
            }
        }
    };
}

deserialize_other!(ByteArrayDeserializer visit_bytearray "bytearray" std::borrow::Cow<'de, [u8]>);
deserialize_other!(IntArrayDeserializer visit_intarray "intarray" super::Array<'de, 4>);
deserialize_other!(LongArrayDeserializer visit_longarray "longarray" super::Array<'de, 8>);
deserialize_other!(ListByteDeserializer visit_list_byte "list byte" std::borrow::Cow<'de, [i8]>);
deserialize_other!(ListShortDeserializer visit_list_short "list short" super::Array<'de, 2>);
deserialize_other!(ListIntDeserializer visit_list_int "list int" super::Array<'de, 4>);
deserialize_other!(ListLongDeserializer visit_list_long "list long" super::Array<'de, 8>);
