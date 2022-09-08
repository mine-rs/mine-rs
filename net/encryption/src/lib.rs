// This code was heavily inspired by https://github.com/smol-rs/blocking/blob/8d8e25831b98a60ff3d96427271d3f46d9331904/src/lib.rs
// which is licensed under MIT or Apache 2.0.
// We are using the MIT license.

//TODO: Replace the channels with atomics?
#![forbid(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::pedantic)]

use std::collections::VecDeque;
use std::option_env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Condvar;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, available_parallelism};
use std::time::Duration;

use aes::cipher::BlockDecryptMut;
use aes::{cipher::BlockEncryptMut, Aes128};
use cfb8::{cipher::inout::InOutBuf, Decryptor, Encryptor};
use futures_channel::oneshot::Sender;
use once_cell::sync::Lazy;
#[cfg(feature = "tracing")]
use tracing::error;

static THREAD_ID: AtomicUsize = AtomicUsize::new(0);
static THREAD_POOL: Lazy<ThreadPool> = Lazy::new(|| ThreadPool {
    limit: match option_env!("MINERS_ENCRYPTION_THREADS") {
        None => 1 + Into::<usize>::into(available_parallelism().unwrap()) / 3,
        Some(v) => v
            .parse()
            .unwrap_or(1 + Into::<usize>::into(available_parallelism().unwrap()) / 3),
    },
    cvar: Condvar::new(),
    inner: Mutex::new(Inner {
        idle: 0,
        total: 0,
        queue: VecDeque::new(),
    }),
});

struct ThreadPool {
    pub limit: usize,
    pub cvar: Condvar,
    pub inner: Mutex<Inner>,
}

struct Inner {
    pub idle: usize,
    pub total: usize,
    pub queue: VecDeque<Task>,
}

pub enum Task {
    Encrypt(EncryptionTask),
    Decrypt(DecryptionTask),
}

#[derive(Debug)]
pub struct EncryptionTask {
    pub data: Arc<Vec<u8>>,
    pub encryption: *mut Encryption,
    // The sender will tell the caller that the encryption has finished and it's safe to use the raw pointer again
    pub send: Sender<()>,
}
// SAFETY: We make sure we don't get data races by using the channel so this is fine
unsafe impl Send for EncryptionTask {}

#[derive(Debug)]
pub struct Encryption {
    pub encryptor: Encryptor<Aes128>,
    pub buf: Vec<u8>,
}

#[derive(Debug)]
pub struct DecryptionTask {
    pub decryption: *mut Decryption,
    // The sender will tell the caller that the encryption has finished and it's safe to use the raw pointer again
    pub send: Sender<()>,
}
// SAFETY: We make sure we don't get data races by using the channel so this is fine
unsafe impl Send for DecryptionTask {}

#[derive(Debug)]
pub struct Decryption {
    pub decryptor: Decryptor<Aes128>,
    pub buf: Vec<u8>,
    pub data: Vec<u8>,
}

impl ThreadPool {
    unsafe fn schedule(&'static self, task: Task) {
        #[allow(clippy::unwrap_used)]
        let mut inner = self.inner.lock().unwrap();
        inner.queue.push_back(task);
        self.cvar.notify_one();
        self.grow_pool(inner);
    }

    fn main_loop(&'static self) {
        #[allow(clippy::unwrap_used)]
        loop {
            let mut inner = self.inner.lock().unwrap();
            while let Some(task) = inner.queue.pop_front() {
                inner.idle -= 1;
                match task {
                    // SAFETY: The only way to add data to the queue is by using an unsafe function, so we're trusting on the user using that correctly
                    Task::Encrypt(task) => unsafe { encrypt(task) },
                    // SAFETY: ^
                    Task::Decrypt(task) => unsafe { decrypt(task) },
                }
            }
            // TODO: Tune the timeout, this will probably also matter for performance.
            // We probably actually want to make it user configureable.
            let timeout = Duration::from_secs(1);
            #[allow(clippy::unwrap_used)]
            let (lock, res) = self.cvar.wait_timeout(inner, timeout).unwrap();
            inner = lock;

            if res.timed_out() && inner.queue.is_empty() {
                inner.idle -= 1;
                inner.total -= 1;
                break;
            }
        }
    }

    fn grow_pool(&'static self, mut inner: MutexGuard<'static, Inner>) {
        // TODO: Tune the value we multiply inner.idle by, that will probably matter a lot
        while inner.queue.len() > inner.idle * 2 && inner.total < self.limit {
            inner.idle += 1;
            inner.total += 1;

            // Notify all idle threads
            self.cvar.notify_all();

            // Generate a new thread ID
            let id = THREAD_ID.fetch_add(1, Ordering::Relaxed);

            // Spawn a new thread
            #[allow(clippy::unwrap_used)]
            thread::Builder::new()
                // if we get a stack overflow, this will be the problem
                .stack_size(1024)
                .name(format!("miners-net-encryption-{id}"))
                .spawn(|| self.main_loop())
                .unwrap();
        }
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument)]
unsafe fn decrypt(task: DecryptionTask) {
    let decryption = task.decryption;

    let buf_len = (*decryption).buf.len();
    let len = buf_len + (*decryption).data.len();

    // Resize the buffer to hold enough data for the encrypted data
    (*decryption).buf.resize(len, 0);

    // SAFETY: We can use unwrap_unchecked here, because we know the supplied buffer is the same length as the data.
    let (chunks, rest) = InOutBuf::new(&(*decryption).data, &mut (*decryption).buf[buf_len..len])
        .unwrap_unchecked()
        .into_chunks();

    debug_assert!(rest.is_empty());
    (*decryption).decryptor.decrypt_blocks_inout_mut(chunks);
    if task.send.send(()).is_err() {
        // The corresponding readhalf was dropped during decryption.
        // We have to drop the decryptor or there would be a memory leak.
        decryption.drop_in_place();
        #[cfg(feature = "tracing")]
        error!("decryption failed: the writehalf was dropped during decryption!");
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument)]
unsafe fn encrypt(task: EncryptionTask) {
    let encryption = task.encryption;
    let data = task.data;

    let buf_len = (*encryption).buf.len();
    let len = buf_len + data.len();

    // Resize the buffer to hold enough data for the encrypted data
    (*encryption).buf.resize(len, 0);

    #[allow(clippy::unwrap_used)]
    // SAFETY: We can use unwrap_unchecked here, because we know the supplied buffer is the same length as the data.
    let (chunks, rest) = InOutBuf::new(&data, &mut (*encryption).buf[buf_len..len])
        .unwrap_unchecked()
        .into_chunks();

    debug_assert!(rest.is_empty());
    (*encryption).encryptor.encrypt_blocks_inout_mut(chunks);

    if task.send.send(()).is_err() {
        // The corresponding writehalf was dropped during encryption.
        // We have to drop the encryptor or there would be a memory leak.
        encryption.drop_in_place();
        #[cfg(feature = "tracing")]
        error!("encrypting failed: the writehalf was dropped during encryption!");
    }
}

//TODO: rename this function
pub unsafe fn unblock(
    task: Task
) {
    THREAD_POOL.schedule(task);
}
