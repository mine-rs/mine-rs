use std::{
    collections::VecDeque,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures_channel::oneshot::Sender;
use parking_lot::Mutex;

use crate::helpers::encrypt;

type Job = (Vec<u8>, usize, Box<cfb8::Encryptor<aes::Aes128>>);

#[allow(clippy::type_complexity)]
static ENCRYPTION_WORKQUEUE: once_cell::sync::Lazy<
    Mutex<VecDeque<(Job, Sender<(Vec<u8>, Box<cfb8::Encryptor<aes::Aes128>>)>)>>,
> = once_cell::sync::Lazy::new(|| Mutex::new(VecDeque::new()));
static ENCRYPTION_MAX_THREADCOUNT: once_cell::sync::Lazy<usize> =
    once_cell::sync::Lazy::new(|| {
        std::env::var("ENCRYPTION_MAX_THREADCOUNT")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(usize::from)
                    .unwrap_or(4)
            })
    });
static ENCRYPTION_THREADCOUNT: AtomicUsize = AtomicUsize::new(0);
static WORKTHREADS_CONDVAR: parking_lot::Condvar = parking_lot::Condvar::new();

pub async fn request_encryption(
    buf: Vec<u8>,
    enc: Box<cfb8::Encryptor<aes::Aes128>>,
) -> futures_channel::oneshot::Receiver<(Vec<u8>, Box<cfb8::Encryptor<aes::Aes128>>)> {
    let len_from_end = buf.len();
    // SAFETY: len_from_end is valid for sure as it is exactly the
    // buffers length
    unsafe { request_partial_encryption(buf, len_from_end, enc).await }
}

/// # Safety
///
/// this is safe as long as len_from_end is not longer than buf.len()
pub(crate) async unsafe fn request_partial_encryption(
    buf: Vec<u8>,
    len_from_end: usize,
    enc: Box<cfb8::Encryptor<aes::Aes128>>,
) -> futures_channel::oneshot::Receiver<(Vec<u8>, Box<cfb8::Encryptor<aes::Aes128>>)> {
    let (send, recv) = futures_channel::oneshot::channel();

    let mut lock = ENCRYPTION_WORKQUEUE.lock();
    lock.push_back(((buf, len_from_end, enc), send));
    let len = lock.len();
    drop(lock);

    if len <= *ENCRYPTION_MAX_THREADCOUNT
        && len > ENCRYPTION_THREADCOUNT.load(std::sync::atomic::Ordering::Acquire)
    {
        spawn_workthread()
    }

    WORKTHREADS_CONDVAR.notify_one();

    recv
}

fn spawn_workthread() {
    static ID: AtomicUsize = AtomicUsize::new(0);
    std::thread::Builder::new()
        .name(format!(
            "miners-encryption-{}",
            ID.fetch_add(1, Ordering::Relaxed)
        ))
        .stack_size(8192) // This can probably be even less but if we get a stackoverflow this line is probably to blame
        .spawn(|| loop {
            let mut mutex = ENCRYPTION_WORKQUEUE.lock();
            if mutex.is_empty() {
                WORKTHREADS_CONDVAR.wait(&mut mutex);
            }
            if let Some(((mut buf, len_from_end, mut enc), send)) = mutex.pop_front() {
                drop(mutex);
                let start = buf.len() - len_from_end;
                encrypt(&mut buf[start..], &mut enc);
                if send.send((buf, enc)).is_err() {
                    eprintln!("async cancellation in encryption workthread");
                }
            }
        })
        .unwrap();
    ENCRYPTION_THREADCOUNT.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
}
