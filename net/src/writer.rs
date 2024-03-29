use std::io::{self, Write};

use futures_lite::{AsyncWrite, AsyncWriteExt};

use crate::{
    helpers::{encrypt, varint_slice},
    packing::PackedData,
};

pub struct Writer<W> {
    encryptor: Option<Option<Box<cfb8::Encryptor<aes::Aes128>>>>,
    inner: W,
    #[cfg(feature = "workpool")]
    unblock_threshold: u32,
}

impl<W> Writer<W> {
    pub fn new(inner: W) -> Writer<W> {
        Writer {
            encryptor: None,
            inner,
            #[cfg(feature = "workpool")]
            unblock_threshold: crate::DEFAULT_UNBLOCK_THRESHOLD,
        }
    }
    #[cfg(feature = "workpool")]
    pub fn set_unblock_threshold(&mut self, threshold: u32) {
        self.unblock_threshold = threshold;
    }
    pub fn enable_encryption(&mut self, encryptor: impl Into<Box<cfb8::Encryptor<aes::Aes128>>>) {
        self.encryptor = Some(Some(encryptor.into()));
    }
}

impl<W> Writer<W>
where
    W: AsyncWrite + Unpin,
{
    async fn write_varint(&mut self, int: u32) -> io::Result<()> {
        let mut var_buf = [0u8; 5];
        let var_slice = varint_slice(int, &mut var_buf);
        if let Some(encryptor) = &mut self.encryptor {
            let mut encryptor = encryptor.take().ok_or(crate::helpers::AsyncCancelled)?;
            encrypt(var_slice, &mut encryptor);
            self.encryptor = Some(Some(encryptor))
        }
        self.inner.write_all(&*var_slice).await
    }
    pub async fn write<'packed>(&mut self, mut data: PackedData) -> io::Result<()> {
        self.write_varint(data.len()).await?;
        if let Some(encryptor) = &mut self.encryptor {
            let mut encryptor = encryptor.take().ok_or(crate::helpers::AsyncCancelled)?;
            #[cfg(feature = "workpool")]
            let encryptor = if data.len() >= self.unblock_threshold {
                // TODO: Remove std::mem::take shenanigans as it is no longer necessary with the bufpool.
                let taken_buf = std::mem::take(&mut data.0 as &mut Vec<u8>);

                let (taken_buf, mutated_encryptor) =
                    crate::workpool::request_encryption(taken_buf, encryptor)
                        .await
                        .await
                        .expect("encryption task was terminated?");
                *data.0 = taken_buf;
                mutated_encryptor
            } else {
                encrypt(data.get_mut(), &mut encryptor);
                encryptor
            };
            #[cfg(not(feature = "workpool"))]
            encrypt(data.get_mut(), &mut encryptor);
            self.encryptor = Some(Some(encryptor));
        }
        self.inner.write_all(data.get()).await?;
        Ok(())
    }
    pub async fn flush(&mut self) -> io::Result<()> {
        self.inner.flush().await
    }
}

impl<W> Writer<W>
where
    W: Write + Unpin,
{
    fn swrite_varint(&mut self, int: u32) -> io::Result<()> {
        let mut var_buf = [0u8; 5];
        let var_slice = varint_slice(int, &mut var_buf);
        if let Some(encryptor) = &mut self.encryptor {
            let mut encryptor = encryptor.take().ok_or(crate::helpers::AsyncCancelled)?;
            encrypt(var_slice, &mut encryptor);
            self.encryptor = Some(Some(encryptor))
        }
        self.inner.write_all(&*var_slice)
    }
    pub fn swrite(&mut self, mut data: PackedData) -> io::Result<()> {
        self.swrite_varint(data.len())?;
        if let Some(encryptor) = &mut self.encryptor {
            let mut encryptor = encryptor.take().ok_or(crate::helpers::AsyncCancelled)?;

            encrypt(data.get_mut(), &mut encryptor);
            self.encryptor = Some(Some(encryptor));
        }
        self.inner.write_all(data.get())?;
        Ok(())
    }
    pub fn sflush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
