mod impls;

use std::borrow::Cow;
use std::fmt::{self, Debug, Display};

use crate::NbtTag;

// #[cfg(feature = "std")]
#[doc(no_inline)]
pub use std::error::Error as StdError;
// #[cfg(not(feature = "std"))]
// #[doc(no_inline)]
// pub use std_error::Error as StdError;

////////////////////////////////////////////////////////////////////////////////

macro_rules! declare_error_trait {
    (Error: Sized $(+ $($supertrait:ident)::+)*) => {
        /// The `Error` trait allows `Deserialize` implementations to create descriptive
        /// error messages belonging to the `Deserializer` against which they are
        /// currently running.
        ///
        /// Every `Deserializer` declares an `Error` type that encompasses both
        /// general-purpose deserialization errors as well as errors specific to the
        /// particular deserialization format. For example the `Error` type of
        /// `serde_json` can represent errors like an invalid JSON escape sequence or an
        /// unterminated string literal, in addition to the error cases that are part of
        /// this trait.
        ///
        /// Most deserializers should only need to provide the `Error::custom` method
        /// and inherit the default behavior for the other methods.
        ///
        /// # Example implementation
        ///
        /// The [example data format] presented on the website shows an error
        /// type appropriate for a basic JSON data format.
        ///
        /// [example data format]: https://serde.rs/data-format.html
        pub trait Error: Sized $(+ $($supertrait)::+)* {
            /// Raised when there is general error when deserializing a type.
            ///
            /// The message should not be capitalized and should not end with a period.
            ///
            /// ```edition2018
            /// # use std::str::FromStr;
            /// #
            /// # struct IpAddr;
            /// #
            /// # impl FromStr for IpAddr {
            /// #     type Err = String;
            /// #
            /// #     fn from_str(_: &str) -> Result<Self, String> {
            /// #         unimplemented!()
            /// #     }
            /// # }
            /// #
            /// use serde::de::{self, Deserialize, Deserializer};
            ///
            /// impl<'de> Deserialize<'de> for IpAddr {
            ///     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            ///     where
            ///         D: Deserializer<'de>,
            ///     {
            ///         let s = String::deserialize(deserializer)?;
            ///         s.parse().map_err(de::Error::custom)
            ///     }
            /// }
            /// ```
            fn custom<T>(msg: T) -> Self
            where
                T: Display;

            /// Raised when a `Deserialize` receives a type different from what it was
            /// expecting.
            ///
            /// The `unexp` argument provides information about what type was received.
            /// This is the type that was present in the input file or other source data
            /// of the Deserializer.
            ///
            /// The `exp` argument provides information about what type was being
            /// expected. This is the type that is written in the program.
            ///
            /// For example if we try to deserialize a String out of a JSON file
            /// containing an integer, the unexpected type is the integer and the
            /// expected type is the string.
            #[cold]
            fn invalid_type(unexp: Unexpected, exp: &dyn Expected) -> Self {
                Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
            }

            /// Raised when a `Deserialize` receives a value of the right type but that
            /// is wrong for some other reason.
            ///
            /// The `unexp` argument provides information about what value was received.
            /// This is the value that was present in the input file or other source
            /// data of the Deserializer.
            ///
            /// The `exp` argument provides information about what value was being
            /// expected. This is the type that is written in the program.
            ///
            /// For example if we try to deserialize a String out of some binary data
            /// that is not valid UTF-8, the unexpected value is the bytes and the
            /// expected value is a string.
            #[cold]
            fn invalid_value(unexp: Unexpected, exp: &dyn Expected) -> Self {
                Error::custom(format_args!("invalid value: {}, expected {}", unexp, exp))
            }

            /// Raised when deserializing a sequence or map and the input data contains
            /// too many or too few elements.
            ///
            /// The `len` argument is the number of elements encountered. The sequence
            /// or map may have expected more arguments or fewer arguments.
            ///
            /// The `exp` argument provides information about what data was being
            /// expected. For example `exp` might say that a tuple of size 6 was
            /// expected.
            #[cold]
            fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
                Error::custom(format_args!("invalid length {}, expected {}", len, exp))
            }

            /// Raised when a `Deserialize` enum type received a variant with an
            /// unrecognized name.
            #[cold]
            fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
                if expected.is_empty() {
                    Error::custom(format_args!(
                        "unknown variant `{}`, there are no variants",
                        variant
                    ))
                } else {
                    Error::custom(format_args!(
                        "unknown variant `{}`, expected {}",
                        variant,
                        OneOf { names: expected }
                    ))
                }
            }

            /// Raised when a `Deserialize` struct type received a field with an
            /// unrecognized name.
            #[cold]
            fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
                if expected.is_empty() {
                    Error::custom(format_args!(
                        "unknown field `{}`, there are no fields",
                        field
                    ))
                } else {
                    Error::custom(format_args!(
                        "unknown field `{}`, expected {}",
                        field,
                        OneOf { names: expected }
                    ))
                }
            }

            /// Raised when a `Deserialize` struct type expected to receive a required
            /// field with a particular name but that field was not present in the
            /// input.
            #[cold]
            fn missing_field(field: &'static str) -> Self {
                Error::custom(format_args!("missing field `{}`", field))
            }

            /// Raised when a `Deserialize` struct type received more than one of the
            /// same field.
            #[cold]
            fn duplicate_field(field: &'static str) -> Self {
                Error::custom(format_args!("duplicate field `{}`", field))
            }
        }
    }
}

// #[cfg(feature = "std")]
declare_error_trait!(Error: Sized + StdError);

// #[cfg(not(feature = "std"))]
// declare_error_trait!(Error: Sized + Debug + Display);

#[derive(Copy, Clone, Debug)]
pub enum Unexpected<'a> {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'a [u8]),
    String(&'a str),
    List,
    Compound,
    IntArray(&'a Array<'a, 4>),
    LongArray(&'a Array<'a, 8>),
}

impl<'a> fmt::Display for Unexpected<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::Unexpected::*;
        match *self {
            Byte(b) => write!(formatter, "byte `{}`", b),
            Short(s) => write!(formatter, "short `{}`", s),
            Int(i) => write!(formatter, "int `{}`", i),
            Long(l) => write!(formatter, "long `{}`", l),
            Float(f) => write!(formatter, "float `{}`", f),
            Double(d) => write!(formatter, "double `{}`", d),
            ByteArray(b) => write!(formatter, "byte array {:?}", b),
            String(s) => write!(formatter, "string {:?}", s),
            List => write!(formatter, "list"),
            Compound => write!(formatter, "compound"),
            IntArray(i) => write!(formatter, "int array {:?}", i),
            LongArray(l) => write!(formatter, "long array {:?}", l),
        }
    }
}

/// `Expected` represents an explanation of what data a `Visitor` was expecting
/// to receive.
///
/// This is used as an argument to the `invalid_type`, `invalid_value`, and
/// `invalid_length` methods of the `Error` trait to build error messages. The
/// message should be a noun or noun phrase that completes the sentence "This
/// Visitor expects to receive ...", for example the message could be "an
/// integer between 0 and 64". The message should not be capitalized and should
/// not end with a period.
///
/// Within the context of a `Visitor` implementation, the `Visitor` itself
/// (`&self`) is an implementation of this trait.
///
/// ```
/// # use std::fmt;
/// #
/// # use miners_nbt::de::{self, Visitor};
/// # use miners_nbt::value::Value;
/// #
/// # struct Example;
/// #
/// # impl<'de> Visitor<'de> for Example {
/// #     type Value = ();
/// #
/// #     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
/// #         write!(formatter, "definitely not a boolean")
/// #     }
/// #
/// fn visit_byte<E>(self, v: i8) -> Result<Self::Value, E>
/// where
///     E: de::Error,
/// {
///     Err(de::Error::invalid_type(Unexpected::Byte(v), &self))
/// }
/// # }
/// ```
///
/// Outside of a `Visitor`, `&"..."` can be used.
///
/// ```
/// # use miners_nbt::de::{self, Unexpected};
/// # use miners_nbt::value::Value;
/// #
/// # fn example<E>() -> Result<(), E>
/// # where
/// #     E: de::Error,
/// # {
/// #     let v = true;
/// return Err(de::Error::invalid_type(Unexpected::Byte(v), &"a string"));
/// # }
/// ```
pub trait Expected {
    /// Format an explanation of what data was being expected. Same signature as
    /// the `Display` and `Debug` traits.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result;
}

impl<'de, T> Expected for T
where
    T: Visitor<'de>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.expecting(formatter)
    }
}

impl<'a> Expected for &'a str {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self)
    }
}

impl<'a> Display for dyn Expected + 'a {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Expected::fmt(self, formatter)
    }
}

/// Used in error messages.
///
/// - expected `a`
/// - expected `a` or `b`
/// - expected one of `a`, `b`, `c`
///
/// The slice of names must not be empty.
struct OneOf {
    names: &'static [&'static str],
}

impl Display for OneOf {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.names.len() {
            0 => panic!(), // special case elsewhere
            1 => write!(formatter, "`{}`", self.names[0]),
            2 => write!(formatter, "`{}` or `{}`", self.names[0], self.names[1]),
            _ => {
                write!(formatter, "one of ")?;
                for (i, alt) in self.names.iter().enumerate() {
                    if i > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "`{}`", alt)?;
                }
                Ok(())
            }
        }
    }
}

pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    #[doc(hidden)]
    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        // Default implementation just delegates to `deserialize` impl.
        *place = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}

pub trait Deserializer<'de>: Sized {
    /// The error type that can be returned if some error occurs during
    /// deserialization.
    type Error: Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;
}

pub trait Visitor<'de>: Sized {
    /// The value produced by this visitor.
    type Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result;

    /// `Value::Byte`
    ///
    /// The default implementation fails with a type error.
    fn visit_byte<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::Byte(v), &self))
    }

    /// `Value::Short`
    ///
    /// The default implementation fails with a type error.
    fn visit_short<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::Short(v), &self))
    }

    /// `Value::Int`
    ///
    /// The default implementation fails with a type error.
    fn visit_int<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::Int(v), &self))
    }

    /// `Value::Long`
    ///
    /// The default implementation fails with a type error.
    fn visit_long<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::Long(v), &self))
    }

    /// `Value::Float`
    ///
    /// The default implementation fails with a type error.
    fn visit_float<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::Float(v), &self))
    }

    /// `Value::Double`
    ///
    /// The default implementation fails with a type error.
    fn visit_double<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::Double(v), &self))
    }

    /// `Value::ByteArray`
    ///
    /// The default implementation fails with a type error.
    fn visit_bytearray<E>(self, v: Cow<'de, [u8]>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::ByteArray(&v), &self))
    }

    /// `Value::String`
    ///
    /// The default implementation fails with a type error.
    fn visit_string<E>(self, v: Cow<'de, str>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::String(&v), &self))
    }

    /// `Value::List(List::Byte)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_byte<E>(self, v: Cow<'de, [i8]>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = v;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::Short)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_short<E>(self, v: Array<'de, 2>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = v;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::Int)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_int<E>(self, v: Array<'de, 4>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = v;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::Long)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_long<E>(self, v: Array<'de, 8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = v;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::Float)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_float<E>(self, v: Cow<'de, [f32]>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = v;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::Double)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_double<E>(self, v: Cow<'de, [f64]>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        _ = v;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::ByteArray)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_bytearray<L>(self, list: L) -> Result<Self::Value, L::Error>
    where
        L: ListAccess<'de, Cow<'de, [u8]>>,
    {
        let _ = list;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::String)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_string<L>(self, list: L) -> Result<Self::Value, L::Error>
    where
        L: ListAccess<'de, Cow<'de, str>>,
    {
        let _ = list;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::List)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_list<L, BA, S, C, IA, LA>(self, list: L) -> Result<Self::Value, L::Error>
    where
        L: ListAccess<'de, ListAccessor<'de, BA, S, C, IA, LA>>,
        BA: ListAccess<'de, Cow<'de, [u8]>>,
        S: ListAccess<'de, Cow<'de, str>>,
        C: CompoundAccess<'de>,
        IA: ListAccess<'de, Array<'de, 4>>,
        LA: ListAccess<'de, Array<'de, 8>>,
    {
        let _ = list;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::Compound)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_compound<L, C>(self, list: L) -> Result<Self::Value, L::Error>
    where
        L: ListAccess<'de, C>,
        C: CompoundAccess<'de>,
    {
        let _ = list;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::IntArray)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_intarray<L>(self, list: L) -> Result<Self::Value, L::Error>
    where
        L: ListAccess<'de, Array<'de, 4>>,
    {
        let _ = list;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::List(List::LongArray)`
    ///
    /// The default implementation fails with a type error.
    fn visit_list_longarray<L>(self, list: L) -> Result<Self::Value, L::Error>
    where
        L: ListAccess<'de, Array<'de, 8>>,
    {
        let _ = list;
        Err(Error::invalid_type(Unexpected::List, &self))
    }

    /// `Value::Compound`
    ///
    /// The default implementation fails with a type error.
    fn visit_compound<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: CompoundAccess<'de>,
    {
        let _ = map;
        Err(Error::invalid_type(Unexpected::Compound, &self))
    }

    /// `Value::IntArray`
    ///
    /// The default implementation fails with a type error.
    fn visit_intarray<E>(self, v: Array<'de, 4>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::IntArray(&v), &self))
    }

    /// `Value::LongArray`
    ///
    /// The default implementation fails with a type error.
    fn visit_longarray<E>(self, v: Array<'de, 8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_type(Unexpected::LongArray(&v), &self))
    }
}

pub trait ListAccess<'de, T> {
    /// The error type that can be returned if some error occurs during
    /// deserialization.
    type Error: Error;

    /// This returns `Ok(Some(value))` for the next value in the sequence, or
    /// `Ok(None)` if there are no more remaining items.
    ///
    /// `Deserialize` implementations should typically use
    /// `SeqAccess::next_element` instead.
    fn next_element_seed(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>;

    /// This returns `Ok(Some(value))` for the next value in the sequence, or
    /// `Ok(None)` if there are no more remaining items.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `SeqAccess` implementations should not override the default behavior.
    fn next_element(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>;

    /// Returns the number of elements remaining in the sequence, if known.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

pub trait CompoundAccess<'de> {
    type Error: Error;
    type Data;

    #[allow(clippy::type_complexity)]
    fn next_key<'a>(
        &'a mut self,
    ) -> Result<Option<(Cow<'de, str>, Field<'a, 'de, Self>)>, Self::Error>;

    fn value<'a, V>(field: Field<'a, 'de, Self>) -> Result<V, Self::Error>
    where
        V: Deserialize<'de>;
    fn consume_field<'a>(field: Field<'a, 'de, Self>) -> Result<(), Self::Error>;
}
pub struct NbtReader<'a> {
    rdr: std::io::Cursor<&'a [u8]>,
}
impl<'a> Drop for NbtReader<'a> {
    fn drop(&mut self) {
        // consume all keys
        while let Ok(Some(_)) = self.next_key() {}
        <NbtTag as ::miners_encoding::Decode>::decode(&mut self.rdr);
    }
}
impl<'a> NbtReader<'a> {
    pub fn new(
        mut rdr: std::io::Cursor<&'a [u8]>,
    ) -> Result<(Cow<'a, str>, Self), ::miners_encoding::decode::Error> {
        if <u8 as ::miners_encoding::Decode>::decode(&mut rdr)? != NbtTag::Compound as u8 {
            return Err(::miners_encoding::decode::Error::InvalidId);
        };
        let name =
            <::miners_encoding::attrs::Mutf8<_> as ::miners_encoding::Decode>::decode(&mut rdr)?
                .into_inner();
        Ok((name, NbtReader { rdr }))
    }
}
impl<'de> CompoundAccess<'de> for NbtReader<'de> {
    type Error = ::miners_encoding::decode::Error;
    type Data = NbtTag;

    fn next_key<'a>(
        &'a mut self,
    ) -> Result<Option<(Cow<'de, str>, Field<'a, 'de, Self>)>, Self::Error> {
        // peek next byte
        let byte = self
            .rdr
            .get_ref()
            .get(self.rdr.position() as usize + 1)
            .ok_or(::miners_encoding::decode::Error::UnexpectedEndOfSlice)?;
        let tag = NbtTag::try_from(*byte)?;
        if let NbtTag::End = tag {
            // early return if end, end gets consumed by destructer
            return Ok(None);
        };
        // otherwise increase counter
        self.rdr.set_position(self.rdr.position() + 1);
        let key: Cow<str> =
            <::miners_encoding::attrs::Mutf8<_> as ::miners_encoding::Decode>::decode(
                &mut self.rdr,
            )?
            .into_inner();

        Ok(Some((
            key,
            Field {
                access: self,
                data: tag,
            },
        )))
    }

    fn value<'a, V>(field: Field<'a, 'de, Self>) -> Result<V, Self::Error>
    where
        V: Deserialize<'de>,
    {
        match field.data {
            NbtTag::End => unreachable!(),
            NbtTag::Byte => V::deserialize(ByteDeserializer::from(
                <i8 as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?,
            )),
            NbtTag::Short => todo!(),
            NbtTag::Int => todo!(),
            NbtTag::Long => todo!(),
            NbtTag::Float => todo!(),
            NbtTag::Double => todo!(),
            NbtTag::ByteArray => todo!(),
            NbtTag::String => todo!(),
            NbtTag::List => {
                match <NbtTag as ::miners_encoding::Decode>::decode(&mut field.access.rdr)? {
                    NbtTag::End => todo!(),
                    NbtTag::Byte => {
                        let bytes = &<&::miners_encoding::attrs::Counted<[u8], i32> as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?.inner;
                        V::deserialize(ListByteDeserializer::from(Cow::Borrowed(unsafe {
                            let (data, len) = (bytes.as_ptr(), bytes.len());
                            std::slice::from_raw_parts(data as *const i8, len)
                        })))
                    }
                    NbtTag::Short => todo!(),
                    NbtTag::Int => todo!(),
                    NbtTag::Long => todo!(),
                    NbtTag::Float => todo!(),
                    NbtTag::Double => todo!(),
                    NbtTag::ByteArray => todo!(),
                    NbtTag::String => todo!(),
                    NbtTag::List => todo!(),
                    NbtTag::Compound => todo!(),
                    NbtTag::IntArray => todo!(),
                    NbtTag::LongArray => todo!(),
                }
            }
            NbtTag::Compound => todo!(),
            NbtTag::IntArray => todo!(),
            NbtTag::LongArray => todo!(),
        }
    }

    fn consume_field<'a>(field: Field<'a, 'de, Self>) -> Result<(), Self::Error> {
        match field.data {
            NbtTag::End => unreachable!(),
            NbtTag::Byte => {
                <i8 as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::Short => {
                <i16 as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::Int => {
                <i32 as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::Long => {
                <i64 as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::Float => {
                <f32 as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::Double => {
                <f64 as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::ByteArray => {
                <&::miners_encoding::attrs::Counted<[u8], i32> as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::String => {
                <::miners_encoding::attrs::Mutf8<Cow<'a, str>> as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::List => {
                todo!()
            }
            NbtTag::Compound => todo!(),
            NbtTag::IntArray => {
                <&::miners_encoding::attrs::Counted<[f32], i32> as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
            NbtTag::LongArray => {
                <&::miners_encoding::attrs::Counted<[f64], i32> as ::miners_encoding::Decode>::decode(&mut field.access.rdr)?;
            }
        }
        Ok(())
    }
}

macro_rules! sv_deser {
    ($($ident:ident$(<$($lt:lifetime),*>)? $visit_fn:ident $val_t:ty);* $(;)?) => {$(
        struct $ident<$($($lt),*,)?E>($val_t, core::marker::PhantomData<E>);
        impl<'de, $($($lt: 'de),*,)? E> Deserializer<'de> for $ident<$($($lt),*,)?E>
        where
            E: Error,
        {
            type Error = E;
            fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                visitor.$visit_fn(self.0)
            }
        }
        impl<$($($lt),*,)?E> From<$val_t> for $ident<$($($lt),*,)?E> {
            fn from(val: $val_t) -> Self {
                $ident(val, core::marker::PhantomData)
            }
        }
    )*};
}
sv_deser! {
    ByteDeserializer visit_byte i8;
    ShortDeserializer visit_short i16;
    IntDeserializer visit_int i32;
    LongDeserializer visit_long i64;
    FloatDeserializer visit_float f32;
    DoubleDeserializer visit_double f64;
    ByteArrayDeserializer<'a> visit_bytearray Cow<'a, [u8]>;
    StringDeserializer<'a> visit_string Cow<'a, str>;
    ListByteDeserializer<'a> visit_list_byte Cow<'a, [i8]>;
    ListShortDeserializer<'a> visit_list_short Array<'a, 2>;
    ListIntDeserializer<'a> visit_list_int Array<'a, 4>;
    ListLongDeserializer<'a> visit_list_long Array<'a, 8>;
    ListFloatDeserializer<'a> visit_list_float Cow<'a, [f32]>;
    ListDoubleDeserializer<'a> visit_list_double Cow<'a, [f64]>;

    IntArrayDeserializer<'a> visit_intarray Array<'a, 4>;
    LongArrayDeserializer<'a> visit_longarray Array<'a, 8>;
}

#[test]
fn a() {
    let rdr = std::io::Cursor::new(&[10u8, 0, 0, 1, 0, 0, 0][..]);

    let (name, mut reader) = NbtReader::new(rdr).unwrap();
    loop {
        let Ok(nxt) = reader.next_key() else {return};
        let Some((key, deser)) = nxt else {break};
        let val = deser.value::<i8>();
        match val {
            Ok(x) => {
                println!("{x:?}");
            }
            Err(e) => {
                panic!("{e:?}")
            }
        }
        eprintln!("{key}");
    }
}

pub struct Field<'a, 'de, A: ?Sized>
where
    A: CompoundAccess<'de>,
{
    access: &'a mut A,
    data: <A as CompoundAccess<'de>>::Data,
}
impl<'a, 'de, A: ?Sized> Drop for Field<'a, 'de, A>
where
    A: CompoundAccess<'de>,
{
    fn drop(&mut self) {
        // self.access.
        todo!()
    }
}
impl<'a, 'de, A> Field<'a, 'de, A>
where
    A: CompoundAccess<'de>,
{
    pub fn value<V>(self) -> Result<V, A::Error>
    where
        V: Deserialize<'de>,
    {
        A::value(self)
    }
}

impl Error for ::miners_encoding::decode::Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        ::miners_encoding::decode::Error::Custom("custom error")
    }
}

pub trait CompoundFieldAccess<'de>: Sized {
    type Error: Error;
    fn value_seed<V>(self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>;
    #[inline]
    fn value<V>(self) -> Result<V, Self::Error>
    where
        V: Deserialize<'de>,
    {
        self.value_seed(core::marker::PhantomData)
    }
}

// pub trait CompoundAccess2<'de> {
//     type Field: CompoundFieldAccess<'de>;

//     #[inline]
//     #[allow(clippy::type_complexity)]
//     fn next_entry_seed<V>(
//         &mut self,
//         vseed: V,
//     ) -> Result<Option<(Cow<'de, str>, V::Value)>, <Self::Field as CompoundFieldAccess<'de>>::Error>
//     where
//         V: DeserializeSeed<'de>,
//     {
//         match self.next_key()? {
//             Some((key, deserializer)) => {
//                 let value = deserializer.value_seed(vseed)?;
//                 Ok(Some((key, value)))
//             }
//             None => Ok(None),
//         }
//     }

//     /// This returns `Ok(Some(key))` for the next key in the map, or `Ok(None)`
//     /// if there are no more remaining entries.
//     ///
//     /// This method exists as a convenience for `Deserialize` implementations.
//     /// `CompoundAccess` implementations should not override the default behavior.
//     #[allow(clippy::type_complexity)]
//     fn next_key(
//         &mut self,
//     ) -> Result<
//         Option<(Cow<'de, str>, Self::Field)>,
//         <Self::Field as CompoundFieldAccess<'de>>::Error,
//     >;

//     /// This returns `Ok(Some((key, value)))` for the next (key-value) pair in
//     /// the map, or `Ok(None)` if there are no more remaining items.
//     ///
//     /// This method exists as a convenience for `Deserialize` implementations.
//     /// `CompoundAccess` implementations should not override the default behavior.
//     #[inline]
//     #[allow(clippy::type_complexity)]
//     fn next_entry<V>(
//         &mut self,
//     ) -> Result<Option<(Cow<'de, str>, V)>, <Self::Field as CompoundFieldAccess<'de>>::Error>
//     where
//         V: Deserialize<'de>,
//     {
//         self.next_entry_seed(core::marker::PhantomData)
//     }

//     /// Returns the number of entries remaining in the map, if known.
//     #[inline]
//     fn size_hint(&self) -> Option<usize> {
//         None
//     }
// }

impl<'de, T> DeserializeSeed<'de> for core::marker::PhantomData<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer)
    }
}

pub trait DeserializeSeed<'de>: Sized {
    /// The type produced by using this seed.
    type Value;

    /// Equivalent to the more common `Deserialize::deserialize` method, except
    /// with some initial piece of data (the seed) passed in.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>;
}

pub enum ListAccessor<'de, BA, S, C, IA, LA> {
    Byte(Cow<'de, [i8]>),
    Short(Array<'de, 2>),
    Int(Array<'de, 4>),
    Long(Array<'de, 8>),
    Float(Cow<'de, [f32]>),
    Double(Cow<'de, [f64]>),
    ByteArray(BA), // ListAccess<'de, Cow<'de, [u8]>>
    String(S),     // ListAccess<'de, Cow<'de, str>>
    List(Box<ListAccessor<'de, BA, S, C, IA, LA>>),
    Compound(C),   // CompoundAccess<'de>
    IntArray(IA),  // ListAccess<'de, Array<i32>>
    LongArray(LA), // ListAccess<'de, Array<i64>>
    Invalid,
}

/// A helper type representing either a reference to a slice of big-endian numbers
/// or a vec of system-endian numbers, allows for optimizations on big-endian systems
pub enum Array<'a, const N: usize> {
    Borrowed {
        ptr: *const [u8; N],
        length: usize,
        lt: core::marker::PhantomData<&'a ()>,
    },
    Owned {
        ptr: *mut [u8; N],
        capacity: usize,
        len: usize,
    },
}

macro_rules! array_debug {
    ($name:literal $n:literal) => {
        impl<'a> Debug for Array<'a, $n> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let (ptr, len): (*const [u8; $n], usize) = match self {
                    #[cfg(not(target_endian = "big"))]
                    Self::Borrowed { .. } => {
                        return f.write_str("BigEndian");
                    }
                    #[cfg(target_endian = "big")]
                    Self::Borrowed { ptr, length, .. } => (*ptr, *length),
                    Self::Owned { ptr, len, .. } => (*ptr, *len),
                };
                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                f.write_str($name)?;
                f.debug_list().entries(slice).finish()
            }
        }
    };
}

array_debug!("ShortArray" 2);
array_debug!("IntArray" 4);
array_debug!("LongArray" 8);

impl<'a, const N: usize> Array<'a, N> {
    pub fn into_system_endian_cow(self) -> Cow<'a, [[u8; N]]> {
        match self {
            Array::Borrowed { ptr, length, .. } => {
                let slice: &[[u8; N]] = unsafe { core::slice::from_raw_parts(ptr, length) };
                let cow;
                #[cfg(not(target_endian = "big"))]
                {
                    let mut vec = Vec::with_capacity(length);
                    for elem in slice {
                        let mut x = *elem;
                        x.reverse();
                        vec.push(x);
                    }
                    cow = Cow::Owned(vec);
                }
                #[cfg(target_endian = "big")]
                {
                    cow = Cow::Borrowed(slice);
                }
                cow
            }
            Array::Owned { ptr, capacity, len } => {
                let vec = unsafe { Vec::from_raw_parts(ptr, len, capacity) };
                Cow::Owned(vec)
            }
        }
    }
    pub fn into_array_endian_cow(&self) -> Cow<'a, [[u8; N]]> {
        match self {
            Array::Borrowed { ptr, length, lt } => {
                Cow::Borrowed(unsafe { ::core::slice::from_raw_parts(*ptr, *length) })
            }
            Array::Owned { ptr, capacity, len } => {
                #[cfg(not(target_endian = "big"))]
                {
                    let slice: &[[u8; N]] = unsafe { ::core::slice::from_raw_parts(*ptr, *len) };
                    let mut vec = Vec::with_capacity(*len);
                    for elem in slice {
                        let mut x = *elem;
                        x.reverse();
                        vec.push(x);
                    }
                    Cow::Owned(vec)
                }
                #[cfg(target_endian = "big")]
                {
                    Cow::Borrowed(unsafe { ::core::slice::from_raw_parts(*ptr, *len) })
                }
            }
        }
    }
}
impl<'a, const N: usize> ::miners_to_static::ToStatic for Array<'a, N> {
    type Static = Array<'static, N>;

    fn to_static(&self) -> Self::Static {
        let vec = match self {
            Array::Borrowed { ptr, length, .. } => {
                let slice: &[[u8; N]] = unsafe { core::slice::from_raw_parts(*ptr, *length) };
                #[cfg(not(target_endian = "big"))]
                {
                    let mut vec = Vec::with_capacity(*length);
                    for elem in slice {
                        let mut x = *elem;
                        x.reverse();
                        vec.push(x);
                    }
                    vec
                }
                #[cfg(target_endian = "big")]
                {
                    slice.to_vec()
                }
            }
            Array::Owned { ptr, len, .. } => {
                let slice: &[[u8; N]] = unsafe { core::slice::from_raw_parts(*ptr, *len) };
                slice.to_vec()
            }
        };
        let mut vec = core::mem::ManuallyDrop::new(vec);
        Array::Owned {
            ptr: vec.as_mut_ptr(),
            capacity: vec.capacity(),
            len: vec.len(),
        }
    }

    fn into_static(self) -> Self::Static {
        let vec = match self.into_system_endian_cow() {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        let mut vec = core::mem::ManuallyDrop::new(vec);
        Array::Owned {
            ptr: vec.as_mut_ptr(),
            capacity: vec.capacity(),
            len: vec.len(),
        }
    }
}
impl<'a, const N: usize> Drop for Array<'a, N> {
    fn drop(&mut self) {
        if let Array::Owned {
            ptr, capacity, len, ..
        } = self
        {
            unsafe {
                Vec::from_raw_parts(*ptr, *len, *capacity);
            }
        }
    }
}

macro_rules! arrayfrom {
    ($($t:ident $num:literal);* $(;)?) => {$(
        impl From<Vec<$t>> for Array<'_, $num> {
            fn from(value: Vec<$t>) -> Self {
                let mut d = ::core::mem::ManuallyDrop::new(value);
                Array::Owned {
                    ptr: d.as_mut_ptr() as *mut _,
                    capacity: d.capacity(),
                    len: d.len(),
                }
            }
        }
    )*};
}
arrayfrom! {
    i16 2;
    u16 2;
    i32 4;
    u32 4;
    i64 8;
    u64 8;
}

trait ArrayT<'a, T>
where
    [T]: std::borrow::ToOwned,
{
    fn into_cow(self) -> Cow<'a, [T]>;
}

macro_rules! into_cow {
    ($($t:ident, $n:literal);* $(;)?) => {$(
        impl<'a> ArrayT<'a, $t> for Array<'a, $n> {
            fn into_cow(self) -> Cow<'a, [$t]> {
                match self.into_system_endian_cow() {
                    Cow::Borrowed(slice) => Cow::Borrowed(unsafe {
                        core::slice::from_raw_parts(slice.as_ptr() as *const $t, slice.len())
                    }),
                    Cow::Owned(vec) => {
                        let mut vec = core::mem::ManuallyDrop::new(vec);
                        Cow::Owned(unsafe {
                            Vec::from_raw_parts(vec.as_mut_ptr() as *mut $t, vec.len(), vec.capacity())
                        })
                    }
                }
            }
        }
    )*};
}

into_cow! {
    i16, 2;
    i32, 4;
    i64, 8;
}

// #[derive(Nbt)]
// #[nbt(rename_all = "camelCase")]
// struct BigTest {
//     #[nbt(rename = "nested compound test")]
//     nested: Nested,
//     int_test: i32,
//     byte_test: i8,
//     string_test: String,
//     #[nbt(list, rename = "listTest (long)")]
//     list_test_long: Vec<i64>,
//     double_test: f64,
//     float_test: f32,
//     long_test: i64,
//     list_test: Vec<NameCreatedOn>,
//     #[nbt(
//         array,
//         rename = "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))"
//     )]
//     byte_array_test: Vec<i8>,
//     short_test: i16,
// }
// #[derive(Nbt)]
// struct Nested {
//     egg: NameValue,
//     ham: NameValue,
// }
// #[derive(Nbt)]
// struct NameValue {
//     name: String,
//     value: f32,
// }
// #[derive(Nbt)]
// #[nbt(rename_all = "kebab-case")]
// struct NameCreatedOn {
//     name: String,
//     created_on: i64,
// }

// impl<'de> Deserialize<'de> for NameCreatedOn {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct NameCreatedOnVisitor;

//         impl<'de> Visitor<'de> for NameCreatedOnVisitor {
//             type Value = NameCreatedOn;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("compound")
//             }
//             fn visit_compound<A>(self, map: A) -> Result<Self::Value, A::Error>
//             where
//                 A: CompoundAccess<'de>,
//             {
//                 let field__name = None;
//                 let field__created_on = None;
//                 while let Some((key, value)) = map.next_key()? {
//                     match &key[..] {
//                         "name" => field__name = Some(value.value()?),
//                         "created-on" => field__created_on = Some(value.value()?),
//                         _ => {}
//                     }
//                 }
//                 let Some(field__name) = field__name else {
//                     return Err(Error::missing_field("name"))
//                 };
//                 let Some(field__created_on) = field__created_on else {
//                     return Err(Error::missing_field("created-on"))
//                 };
//                 Ok(NameCreatedOn {
//                     name: field__name,
//                     created_on: field__created_on,
//                 })
//             }
//         }

//         deserializer.deserialize(NameCreatedOnVisitor)
//     }
// }
