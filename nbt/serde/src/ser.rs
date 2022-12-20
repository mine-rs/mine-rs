#[repr(transparent)]
pub struct Serializer<W> {
    writer: W,
}
#[repr(transparent)]
pub struct TagSerializer<W> {
    writer: W
}
#[repr(transparent)]
pub struct CompoundSerializer<W> {
    writer: W,
}

macro_rules! error_fns {
    (Err($err:path); $( $fn_name:ident $ty:ty ),*) => {
        $(
            fn $fn_name(self, _v: $ty) -> Result<Self::Ok, Self::Error> {
                Err($err)
            }
        )*
    };
}

impl<'w, W> serde::Serializer for &'w mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = crate::Error;
    type SerializeSeq = serde::ser::Impossible<(), crate::Error>;
    type SerializeTuple = serde::ser::Impossible<(), crate::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<(), crate::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<(), crate::Error>;
    type SerializeMap = &'w mut CompoundSerializer<W>;
    type SerializeStruct = &'w mut CompoundSerializer<W>;
    type SerializeStructVariant = serde::ser::Impossible<(), crate::Error>;

    fn is_human_readable(&self) -> bool {
        false
    }

    error_fns!(Err(crate::Error::NonCompoundRoot);
        serialize_bool bool,
        serialize_i8 i8, serialize_u8 u8,
        serialize_i16 i16, serialize_u16 u16,
        serialize_i32 i32, serialize_u32 u32,
        serialize_i64 i64, serialize_u64 u64,
        serialize_f32 f32, serialize_f64 f64,
        serialize_char char, serialize_str &str,
        serialize_bytes &[u8]
    );

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}
impl<W> serde::ser::SerializeTupleStruct for &mut CompoundSerializer<W> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<W> serde::ser::SerializeMap for &mut CompoundSerializer<W> {
    type Ok = ();

    type Error = crate::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<W> serde::ser::SerializeStruct for &mut CompoundSerializer<W> {
    type Ok = ();

    type Error = crate::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}