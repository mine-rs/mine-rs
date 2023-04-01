use std::fmt::Display;

// #[cfg(feature = "std")]
#[doc(no_inline)]
pub use std::error::Error as StdError;

use crate::{de::Array, NbtTag};
// #[cfg(not(feature = "std"))]
// #[doc(no_inline)]
// pub use std_error::Error as StdError;

////////////////////////////////////////////////////////////////////////////////

macro_rules! declare_error_trait {
    (Error: Sized $(+ $($supertrait:ident)::+)*) => {
        /// Trait used by `Serialize` implementations to generically construct
        /// errors belonging to the `Serializer` against which they are
        /// currently running.
        ///
        /// # Example implementation
        ///
        /// The [example data format] presented on the website shows an error
        /// type appropriate for a basic JSON data format.
        ///
        /// [example data format]: https://serde.rs/data-format.html
        pub trait Error: Sized $(+ $($supertrait)::+)* {
            /// Used when a [`Serialize`] implementation encounters any error
            /// while serializing a type.
            ///
            /// The message should not be capitalized and should not end with a
            /// period.
            ///
            /// For example, a filesystem [`Path`] may refuse to serialize
            /// itself if it contains invalid UTF-8 data.
            ///
            /// ```edition2018
            /// # struct Path;
            /// #
            /// # impl Path {
            /// #     fn to_str(&self) -> Option<&str> {
            /// #         unimplemented!()
            /// #     }
            /// # }
            /// #
            /// use serde::ser::{self, Serialize, Serializer};
            ///
            /// impl Serialize for Path {
            ///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            ///     where
            ///         S: Serializer,
            ///     {
            ///         match self.to_str() {
            ///             Some(s) => serializer.serialize_str(s),
            ///             None => Err(ser::Error::custom("path contains invalid UTF-8 characters")),
            ///         }
            ///     }
            /// }
            /// ```
            ///
            /// [`Path`]: https://doc.rust-lang.org/std/path/struct.Path.html
            /// [`Serialize`]: ../trait.Serialize.html
            fn custom<T>(msg: T) -> Self
            where
                T: Display;
        }
    }
}

// #[cfg(feature = "std")]
declare_error_trait!(Error: Sized + StdError);
// #[cfg(not(feature = "std"))]
// declare_error_trait!(Error: Sized + Debug + Display);

pub trait Serialize<'s> {
    fn serialize<C>(self, serializer: FieldSerializer<'s, '_, '_, C>) -> Result<(), C::Error>
    where
        C: CompoundSerializer<'s>;
}

pub trait CompoundSerializer<'s>: Sized {
    type Error: Error;
    type Key<'a>;
    type SerializeList<'l>: ListSerializer<'l>
    where
        Self: 'l;
    type SerializeCompound<'c>: CompoundSerializer<'c>
    where
        Self: 'c;
    fn serialize_byte(&mut self, key: Self::Key<'_>, v: i8) -> Result<(), Self::Error>;
    fn serialize_short(&mut self, key: Self::Key<'_>, v: i16) -> Result<(), Self::Error>;
    fn serialize_int(&mut self, key: Self::Key<'_>, v: i32) -> Result<(), Self::Error>;
    fn serialize_long(&mut self, key: Self::Key<'_>, v: i64) -> Result<(), Self::Error>;
    fn serialize_float(&mut self, key: Self::Key<'_>, v: f32) -> Result<(), Self::Error>;
    fn serialize_double(&mut self, key: Self::Key<'_>, v: f64) -> Result<(), Self::Error>;
    fn serialize_bytearray(&mut self, key: Self::Key<'_>, v: &[u8]) -> Result<(), Self::Error>;
    fn serialize_string(&mut self, key: Self::Key<'_>, v: &str) -> Result<(), Self::Error>;
    fn serialize_list<'l>(
        &'l mut self,
        key: Self::Key<'_>,
    ) -> Result<Self::SerializeList<'l>, Self::Error>;
    fn serialize_compound<'c>(
        &'c mut self,
        key: Self::Key<'_>,
        len: Option<usize>,
    ) -> Result<Self::SerializeCompound<'c>, Self::Error>;
    fn serialize_intarray(
        &mut self,
        key: Self::Key<'_>,
        v: Array<'_, 4>,
    ) -> Result<(), Self::Error>;
    fn serialize_longarray(
        &mut self,
        key: Self::Key<'_>,
        v: Array<'_, 8>,
    ) -> Result<(), Self::Error>;
    fn end(self) -> Result<(), Self::Error>;
}

pub trait ListSerializer<'s>: Sized {
    type Error: Error;
    // type SerializeList: for<'i> ListSerializer<'i>;
    type ListIter: ListIter;
    type CompoundIter: CompoundIter;
    fn serialize_byte(self, v: &[i8]) -> Result<(), Self::Error>;
    fn serialize_short(self, v: Array<'_, 2>) -> Result<(), Self::Error>;
    fn serialize_int(self, v: Array<'_, 4>) -> Result<(), Self::Error>;
    fn serialize_long(self, v: Array<'_, 8>) -> Result<(), Self::Error>;
    fn serialize_float(self, v: &[f32]) -> Result<(), Self::Error>;
    fn serialize_double(self, v: &[f64]) -> Result<(), Self::Error>;
    fn serialize_bytearray(self, v: &[&[u8]]) -> Result<(), Self::Error>;
    fn serialize_string(self, v: &[&str]) -> Result<(), Self::Error>;
    fn serialize_list(self, n: i32) -> Result<Self::ListIter, Self::Error>;
    fn serialize_compound(self, n: i32) -> Result<Self::CompoundIter, Self::Error>;
    fn serialize_intarray(self, v: &[Array<'_, 4>]) -> Result<(), Self::Error>;
    fn serialize_longarray(self, v: &[Array<'_, 8>]) -> Result<(), Self::Error>;
    fn serialize_invalid(self) -> Result<(), Self::Error>;
}

fn nslice<const N: usize>(i: &[[u8; N]]) -> &[u8] {
    // Safety: this is safe
    unsafe { ::core::slice::from_raw_parts(i.as_ptr() as _, i.len() * N) }
}

impl<'s, 'w, W: ?Sized + std::io::Write> ListSerializer<'s> for ListWriter<'w, W> {
    type Error = ::miners_encoding::encode::Error;

    type ListIter = ListWriterIter<'w, W>;
    type CompoundIter = CompoundWriterIter<'w, W>;

    fn serialize_byte(self, v: &[i8]) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::Byte, &mut writer)?;
        ::miners_encoding::Encode::encode(
            <&::miners_encoding::attrs::Counted<[i8], i32>>::from(v),
            &mut writer,
        )?;
        Ok(())
    }

    fn serialize_short(self, v: Array<'_, 2>) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::Short, &mut writer)?;
        let cow = v.into_array_endian_cow();
        ::miners_encoding::Encode::encode(&i32::try_from(cow.len())?, &mut writer)?;
        writer.write_all(nslice(&cow[..]))?;
        Ok(())
    }

    fn serialize_int(self, v: Array<'_, 4>) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::Int, &mut writer)?;
        let cow = v.into_array_endian_cow();
        ::miners_encoding::Encode::encode(&i32::try_from(cow.len())?, &mut writer)?;
        writer.write_all(nslice(&cow[..]))?;
        Ok(())
    }

    fn serialize_long(self, v: Array<'_, 8>) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::Long, &mut writer)?;
        let cow = v.into_array_endian_cow();
        ::miners_encoding::Encode::encode(&i32::try_from(cow.len())?, &mut writer)?;
        writer.write_all(nslice(&cow[..]))?;
        Ok(())
    }

    fn serialize_float(self, v: &[f32]) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::Float, &mut writer)?;
        ::miners_encoding::Encode::encode(
            &<&::miners_encoding::attrs::Counted<[f32], i32>>::from(v),
            &mut writer,
        )?;
        Ok(())
    }

    fn serialize_double(self, v: &[f64]) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::Double, &mut writer)?;
        ::miners_encoding::Encode::encode(
            &<&::miners_encoding::attrs::Counted<[f64], i32>>::from(v),
            &mut writer,
        )?;
        Ok(())
    }

    fn serialize_bytearray(self, v: &[&[u8]]) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::ByteArray, &mut writer)?;
        ::miners_encoding::Encode::encode(&i32::try_from(v.len())?, &mut writer)?;
        for ba in v {
            ::miners_encoding::Encode::encode(
                <&::miners_encoding::attrs::Counted<[u8], i32>>::from(*ba),
                &mut writer,
            )?;
        }
        Ok(())
    }

    fn serialize_string(self, v: &[&str]) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::String, &mut writer)?;
        ::miners_encoding::Encode::encode(&i32::try_from(v.len())?, &mut writer)?;
        for s in v {
            ::miners_encoding::Encode::encode(
                <::miners_encoding::attrs::Mutf8<str>>::from(*s),
                &mut writer,
            )?;
        }
        Ok(())
    }

    fn serialize_list(self, n: i32) -> Result<Self::ListIter, Self::Error> {
        let writer = self.into_writer();
        ListWriterIter::new(n, writer)
    }

    fn serialize_compound(self, n: i32) -> Result<Self::CompoundIter, Self::Error> {
        let writer = self.into_writer();
        CompoundWriterIter::new(n, writer)
    }

    fn serialize_intarray(self, v: &[Array<'_, 4>]) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::IntArray, &mut writer)?;
        ::miners_encoding::Encode::encode(&i32::try_from(v.len())?, &mut writer)?;
        for ba in v {
            let cow = ba.into_array_endian_cow();
            ::miners_encoding::Encode::encode(&i32::try_from(cow.len())?, &mut writer)?;
            writer.write_all(nslice(&cow[..]))?;
        }
        Ok(())
    }

    fn serialize_longarray(self, v: &[Array<'_, 8>]) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::LongArray, &mut writer)?;
        ::miners_encoding::Encode::encode(&i32::try_from(v.len())?, &mut writer)?;
        for ba in v {
            let cow = ba.into_array_endian_cow();
            ::miners_encoding::Encode::encode(&i32::try_from(cow.len())?, &mut writer)?;
            writer.write_all(nslice(&cow[..]))?;
        }
        Ok(())
    }

    fn serialize_invalid(self) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::End, &mut self.into_writer())
    }
}

pub enum ListOfLists<'v, BA, S, C, IA, LA> {
    Byte(&'v [i8]),
    Short(Array<'v, 2>),
    Int(Array<'v, 4>),
    Long(Array<'v, 8>),
    Float(&'v [f32]),
    Double(&'v [f64]),
    ByteArray(BA),
    String(S),
    List(Box<ListOfLists<'v, BA, S, C, IA, LA>>),
    Compound(C),
    IntArray(IA),
    LongArray(LA),
    Invalid,
}

#[repr(transparent)]
pub struct CompoundWriter<'w, W: ?Sized + std::io::Write> {
    writer: &'w mut W,
}
impl<'w, W: ?Sized + std::io::Write> CompoundWriter<'w, W> {
    pub fn new(
        mut writer: &'w mut W,
        root_tag: &str,
    ) -> Result<Self, ::miners_encoding::encode::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Compound, &mut writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(root_tag),
            &mut writer,
        )?;
        Ok(CompoundWriter { writer })
    }
    fn into_writer(self) -> &'w mut W {
        // Safety: CompoundWriter is repr(transparent)
        unsafe { ::core::mem::transmute(self) }
    }
}
impl<'w, W: ?Sized + std::io::Write> Drop for CompoundWriter<'w, W> {
    fn drop(&mut self) {
        ::miners_encoding::Encode::encode(&NbtTag::End, &mut self.writer);
    }
}
#[repr(transparent)]
pub struct ListWriter<'w, W: ?Sized + std::io::Write> {
    writer: &'w mut W,
}
impl<'w, W: ?Sized + std::io::Write> ListWriter<'w, W> {
    fn into_writer(self) -> &'w mut W {
        // Safety: ListWriter is repr(transparent)
        unsafe { ::core::mem::transmute(self) }
    }
}
impl<'s, 'w, W: ?Sized + std::io::Write> CompoundSerializer<'s> for CompoundWriter<'w, W>
where
    W: std::io::Write,
{
    type Error = ::miners_encoding::encode::Error;

    type Key<'a> = &'a str;

    type SerializeList<'l> = ListWriter<'l, W> where Self: 'l;
    type SerializeCompound<'c> = CompoundWriter<'c, W> where Self: 'c;

    fn serialize_byte(&mut self, key: Self::Key<'_>, v: i8) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Byte, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(&v, &mut self.writer)
    }

    fn serialize_short(&mut self, key: Self::Key<'_>, v: i16) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Short, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(&v, &mut self.writer)
    }

    fn serialize_int(&mut self, key: Self::Key<'_>, v: i32) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Int, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(&v, &mut self.writer)
    }

    fn serialize_long(&mut self, key: Self::Key<'_>, v: i64) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Long, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(&v, &mut self.writer)
    }

    fn serialize_float(&mut self, key: Self::Key<'_>, v: f32) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Float, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(&v, &mut self.writer)
    }

    fn serialize_double(&mut self, key: Self::Key<'_>, v: f64) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Double, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(&v, &mut self.writer)
    }

    fn serialize_bytearray(&mut self, key: Self::Key<'_>, v: &[u8]) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::ByteArray, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(
            <&::miners_encoding::attrs::Counted<_, i32>>::from(v),
            &mut self.writer,
        )
    }

    fn serialize_string(&mut self, key: Self::Key<'_>, v: &str) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::String, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(v),
            &mut self.writer,
        )
    }

    fn serialize_list<'l>(
        &'l mut self,
        key: Self::Key<'_>,
    ) -> Result<Self::SerializeList<'l>, Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::List, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        Ok(ListWriter {
            writer: self.writer,
        })
    }

    fn serialize_compound<'c>(
        &'c mut self,
        key: Self::Key<'_>,
        _len: Option<usize>,
    ) -> Result<Self::SerializeCompound<'c>, Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Compound, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        Ok(CompoundWriter {
            writer: self.writer,
        })
    }

    fn serialize_intarray(
        &mut self,
        key: Self::Key<'_>,
        v: Array<'_, 4>,
    ) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::IntArray, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        let cow = v.into_array_endian_cow();
        ::miners_encoding::Encode::encode(&i32::try_from(cow.len())?, &mut self.writer)?;
        self.writer.write_all(nslice(&cow[..]))?;
        Ok(())
    }

    fn serialize_longarray(
        &mut self,
        key: Self::Key<'_>,
        v: Array<'_, 8>,
    ) -> Result<(), Self::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::LongArray, &mut self.writer)?;
        ::miners_encoding::Encode::encode(
            <::miners_encoding::attrs::Mutf8<str>>::from(key),
            &mut self.writer,
        )?;
        let cow = v.into_array_endian_cow();
        ::miners_encoding::Encode::encode(&i32::try_from(cow.len())?, &mut self.writer)?;
        self.writer.write_all(nslice(&cow[..]))?;
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        let mut writer = self.into_writer();
        ::miners_encoding::Encode::encode(&NbtTag::End, &mut writer)
    }
}

impl<'w, W: ?Sized + std::io::Write> Drop for ListWriter<'w, W> {
    fn drop(&mut self) {
        // write invalid list byte
        ::miners_encoding::Encode::encode(&NbtTag::End, &mut self.writer);
    }
}
pub struct ListWriterIter<'w, W: ?Sized + std::io::Write> {
    num: i32,
    writer: &'w mut W,
}
// impl<'w, W: ?Sized + std::io::Write> IterableSuper for ListWriterIter<'w, W> {
// }
impl<'w, W: ?Sized + std::io::Write> ListIter for ListWriterIter<'w, W> {
    type Error = ::miners_encoding::encode::Error;
    type Item<'i> = ListWriter<'i, W> where Self: 'i;
    #[allow(clippy::needless_lifetimes)]
    fn next<'s>(&'s mut self) -> Option<Self::Item<'s>> {
        if self.num == 0 {
            return None;
        }
        self.num -= 1;
        Some(ListWriter {
            writer: self.writer,
        })
    }
    fn end(mut self) -> Result<(), Self::Error> {
        while let Some(list) = self.next() {
            list.serialize_invalid()?;
        }
        // don't run the destructor
        let _ = self.into_writer();
        Ok(())
    }
}
impl<'w, W: ?Sized + std::io::Write> ListWriterIter<'w, W> {
    fn new(num: i32, mut writer: &'w mut W) -> Result<Self, ::miners_encoding::encode::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::List, &mut writer)?;
        ::miners_encoding::Encode::encode(&num, &mut writer)?;
        Ok(Self { num, writer })
    }
    fn into_writer(self) -> &'w mut W {
        let mut d = ::core::mem::ManuallyDrop::new(self);
        // Safety: this is safe because i say so
        unsafe {
            #[allow(clippy::uninit_assumed_init, invalid_value)]
            let mut ptr = ::core::mem::MaybeUninit::uninit().assume_init();
            ::core::mem::swap(&mut d.writer, &mut ptr);
            ptr
        }
    }
}
impl<'w, W: ?Sized + std::io::Write> Drop for ListWriterIter<'w, W> {
    fn drop(&mut self) {
        // trigger all remaining drop destructors
        while self.next().is_some() {}
        ::miners_encoding::Encode::encode(&NbtTag::End, &mut self.writer);
    }
}
pub trait ListIter {
    type Error: Error;
    type Item<'i>: ListSerializer<'i>
    where
        Self: 'i;
    #[allow(clippy::needless_lifetimes)]
    fn next<'s>(&'s mut self) -> Option<Self::Item<'s>>;
    fn end(self) -> Result<(), Self::Error>;
}
pub trait CompoundIter {
    type Error: Error;
    type Item<'i>: CompoundSerializer<'i>
    where
        Self: 'i;
    #[allow(clippy::needless_lifetimes)]
    fn next<'s>(&'s mut self) -> Option<Self::Item<'s>>;
    fn end(self) -> Result<(), Self::Error>;
}
pub struct CompoundWriterIter<'w, W: ?Sized + std::io::Write> {
    num: i32,
    writer: &'w mut W,
}
impl<'w, W: ?Sized + std::io::Write> CompoundWriterIter<'w, W> {
    fn new(num: i32, mut writer: &'w mut W) -> Result<Self, ::miners_encoding::encode::Error> {
        ::miners_encoding::Encode::encode(&NbtTag::Compound, &mut writer)?;
        ::miners_encoding::Encode::encode(&num, &mut writer)?;
        Ok(Self { num, writer })
    }
    fn into_writer(self) -> &'w mut W {
        let mut d = ::core::mem::ManuallyDrop::new(self);
        // Safety: this is safe because i say so
        unsafe {
            #[allow(clippy::uninit_assumed_init, invalid_value)]
            let mut ptr = ::core::mem::MaybeUninit::uninit().assume_init();
            ::core::mem::swap(&mut d.writer, &mut ptr);
            ptr
        }
    }
}
impl<'w, W: ?Sized + std::io::Write> CompoundIter for CompoundWriterIter<'w, W> {
    type Error = ::miners_encoding::encode::Error;
    type Item<'i> = CompoundWriter<'i, W> where Self: 'i;
    #[allow(clippy::needless_lifetimes)]
    fn next<'s>(&'s mut self) -> Option<Self::Item<'s>> {
        if self.num == 0 {
            return None;
        }
        self.num -= 1;
        Some(CompoundWriter {
            writer: self.writer,
        })
    }
    fn end(mut self) -> Result<(), Self::Error> {
        while let Some(cpd) = self.next() {
            cpd.end()?;
        }
        // don't run the destructor
        let _ = self.into_writer();
        Ok(())
    }
}
impl<'w, W: ?Sized + std::io::Write> Drop for CompoundWriterIter<'w, W> {
    fn drop(&mut self) {
        while self.next().is_some() {}
    }
}

impl Error for ::miners_encoding::encode::Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        ::miners_encoding::encode::Error::Custom("custom error")
    }
}

pub trait CompoundSerializerExt<'s>: CompoundSerializer<'s> {
    fn serialize_entry<V>(&mut self, key: Self::Key<'_>, value: V) -> Result<(), Self::Error>
    where
        V: Serialize<'s>,
        for<'a> Self::Key<'a>: std::marker::Sized,
    {
        value.serialize(FieldSerializer {
            compound: self,
            key,
        })
    }
}
impl<'s, T> CompoundSerializerExt<'s> for T where T: CompoundSerializer<'s> {}

pub struct FieldSerializer<'s, 'c, 'k, C>
where
    C: CompoundSerializer<'s>,
{
    compound: &'c mut C,
    key: C::Key<'k>,
}
macro_rules! field_fwd {
    ($($ident:ident $val_t:ty);* $(;)?) => {$(
        pub fn $ident(self, v: $val_t) -> Result<(), C::Error> {
            self.compound.$ident(self.key, v)
        }
    )*};
}
impl<'s, 'c, 'k, C> FieldSerializer<'s, 'c, 'k, C>
where
    C: CompoundSerializer<'s>,
{
    field_fwd! {
        serialize_byte i8;
        serialize_short i16;
        serialize_int i32;
        serialize_long i64;
        serialize_float f32;
        serialize_double f64;
        serialize_bytearray &[u8];
    }
    pub fn serialize_list(self) -> Result<C::SerializeList<'c>, C::Error> {
        self.compound.serialize_list(self.key)
    }
    pub fn serialize_compound(
        self,
        len: Option<usize>,
    ) -> Result<C::SerializeCompound<'c>, C::Error> {
        self.compound.serialize_compound(self.key, len)
    }
    field_fwd! {
        serialize_intarray Array<'_, 4>;
        serialize_longarray Array<'_, 8>;
    }
}
// test.nbt
fn test2(writer: &mut impl std::io::Write) -> Result<(), ::miners_encoding::encode::Error> {
    let mut root = CompoundWriter::new(writer, "hello world")?;
    root.serialize_string("name", "Bananrama")?;
    root.end()?;
    Ok(())
}
#[allow(rustdoc::all, non_snake_case)]
fn bigtest(writer: &mut impl std::io::Write) -> Result<(), ::miners_encoding::encode::Error> {
    let mut Level = CompoundWriter::new(writer, "Level")?;
    Level.serialize_long("longTest", 9223372036854775807)?;
    Level.serialize_short("shortTest", 32767)?;
    Level.serialize_string("stringTest", "HELLO WORLD THIS IS A TEST STRING ÅÄÖ!")?;
    Level.serialize_float("floatTest", 0.498_231_47)?;
    Level.serialize_int("intTest", 2147483647)?;
    let mut nested_compound_test = Level.serialize_compound("nested compound test", Some(2))?;
    let mut egg = nested_compound_test.serialize_compound("egg", Some(2))?;
    egg.serialize_string("name", "Eggbert")?;
    egg.serialize_float("value", 0.5)?;
    egg.end()?;
    let mut ham = nested_compound_test.serialize_compound("ham", Some(2))?;
    ham.serialize_string("name", "Hampus")?;
    ham.serialize_float("value", 0.75)?;
    ham.end()?;
    nested_compound_test.end()?;
    Level
        .serialize_list("listTest (long)")?
        .serialize_long(vec![11u64, 12, 13, 14, 15].into())?;
    let mut listTest_compound = Level
        .serialize_list("listTest (compound)")?
        .serialize_compound(2)?;
    let mut listTest_compound0 = listTest_compound.next().unwrap();
    listTest_compound0.serialize_string("name", "Compound tag #0")?;
    listTest_compound0.serialize_long("created-on", 1264099775885)?;
    listTest_compound0.end()?;
    let mut listTest_compound1 = listTest_compound.next().unwrap();
    listTest_compound1.serialize_string("name", "Compound tag #1")?;
    listTest_compound1.serialize_long("created-on", 1264099775885)?;
    listTest_compound1.end()?;
    listTest_compound.end()?;
    Level.serialize_byte("byteTest", 127)?;
    Level.serialize_bytearray("byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))", &(0i32..1000).map(|n|((n*n*255+n*7)%100) as u8).collect::<Vec<u8>>())?;
    Level.serialize_double("doubleTest", 0.493_128_713_218_231_5)?;
    Ok(())
}

fn test(writer: &mut impl std::io::Write) -> Result<(), ::miners_encoding::encode::Error> {
    let mut ser = CompoundWriter::new(writer, "root")?;
    ser.serialize_byte("hi", 32)?;
    // ser.serialize_list("idk")?
    //     .serialize_float(&[3.2, 5.2, -12.0])?;
    let mut cpd = ser.serialize_compound("compound", None)?;
    cpd.serialize_int("int", 32)?;
    // ser.serialize_float("float", 3.2)?;
    cpd.serialize_long("long", 64)?;
    drop(cpd);
    let mut listsser = ser.serialize_list("list")?.serialize_list(2)?;
    const WRONG_IMPL: ::miners_encoding::encode::Error =
        ::miners_encoding::encode::Error::Custom("wrong impl");
    let mut compoundser = listsser.next().ok_or(WRONG_IMPL)?.serialize_compound(3)?;
    while let Some(mut cpdser) = compoundser.next() {
        cpdser.serialize_long("nottooshort", -234444444)?;
        cpdser.serialize_float("amplifier", 4.82)?;
    }
    compoundser.end()?;
    listsser
        .next()
        .ok_or(WRONG_IMPL)?
        .serialize_short(vec![-32i16, 423, 53, 25565].into())?;
    // last list value not initialized, list should be invalid with end tag and compound
    // should also be ended automatically
    Ok(())
}

#[test]
fn testit() {
    let mut buf = vec![];
    test(&mut buf).unwrap();
    eprintln!("{:?}", buf);
}

// pub trait ValueSerializer: Sized {
//     type Ok;
//     type Error: Error;
//     fn serialize_byte(self, v: i8) -> Result<Self::Ok, Self::Error>;
//     fn serialize_short(self, v: i16) -> Result<Self::Ok, Self::Error>;
//     fn serialize_int(self, v: i32) -> Result<Self::Ok, Self::Error>;
//     fn serialize_long(self, v: i64) -> Result<Self::Ok, Self::Error>;
//     fn serialize_float(self, v: f32) -> Result<Self::Ok, Self::Error>;
//     fn serialize_double(self, v: f64) -> Result<Self::Ok, Self::Error>;
//     // bytearray
//     fn serialize_string(self, v: &str) -> Result<Self::Ok, Self::Error>;
//     // list
//     // compound
//     // intarray
//     // longarray
// }
