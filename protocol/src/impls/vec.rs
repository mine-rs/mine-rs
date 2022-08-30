use crate::{attrs::Var, *};

// generic Vec<T>

impl<'a, T> ProtocolRead<'a> for Vec<T>
where
    T: ProtocolRead<'a>,
{
    fn read(cursor: &'_ mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len = <Var<u32>>::read(cursor)?.0;
        (0..len).map(|_| T::read(cursor)).collect()
    }
}
impl<T> ProtocolWrite for Vec<T>
where
    T: ProtocolWrite,
{
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len = self.len() as u32;
        <Var<u32>>::write(Var(len), writer)?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }

    fn size_hint() -> usize {
        <Var<u32>>::size_hint()
    }
}
impl<T: ToStatic> ToStatic for Vec<T> {
    type Static = Vec<<T as ToStatic>::Static>;
    fn to_static(&self) -> Self::Static {
        self.iter().map(ToStatic::to_static).collect()
    }
    fn into_static(self) -> Self::Static {
        self.into_iter().map(ToStatic::into_static).collect()
    }
}