use std::{
    cell::UnsafeCell,
    cmp::Ordering,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

static BUFPOOL_MAX_MEMORY: once_cell::sync::Lazy<usize> = once_cell::sync::Lazy::new(|| {
    std::env::var("BUFPOOL_MAX_MEMORY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64 * 1024 * 1024) // 64 MiB
});

thread_local! {
    static BUF_POOL: UnsafeCell<(Vec<Option<Vec<u8>>>, usize)> = UnsafeCell::new((vec![None], 0));
}

pub fn request_buf(req_cap: usize) -> BufGuard {
    BufGuard::new(BUF_POOL.with(|pool| {
        let pool = unsafe { pool.get().as_mut().unwrap() };
        let last = pool.0.len() - 1;
        for (i, v) in pool.0.iter_mut().enumerate() {
            match v {
                Some(u) => {
                    if u.capacity() >= req_cap {
                    } else if i == last {
                        u.reserve(req_cap - u.capacity());
                        pool.1 -= u.capacity();
                        return std::mem::take(v).unwrap();
                    }
                }
                None => continue,
            }
        }
        // No buffers were available.
        pool.0.push(None);
        Vec::with_capacity(req_cap)
    }))
}

fn reorder_pool() {
    BUF_POOL.with(|pool| {
        let pool = unsafe { pool.get().as_mut().unwrap() };
        pool.0.sort_by(|a, b| {
            // Order the vec from least to most capacity.
            match (a, b) {
                (None, None) | (None, _) | (_, None) => Ordering::Equal,
                (Some(a), Some(b)) => a.capacity().cmp(&b.capacity()),
            }
        })
    });
}

fn return_buf(buf: Vec<u8>) {
    BUF_POOL.with(|pool| {
        let pool = unsafe { pool.get().as_mut().unwrap() };
        pool.1 += buf.capacity();
        for i in &mut pool.0 {
            if i.is_none() {
                *i = Some(buf);
                reorder_pool();
                let mut j: Option<usize> = None;
                loop {
                    if &pool.1 > &BUFPOOL_MAX_MEMORY {
                        match j {
                            None => {
                                // Find the largest buffer
                                j = Some(
                                    pool.0
                                        .iter()
                                        .rev()
                                        .enumerate()
                                        .find(|v| if let Some(_) = v.1 { true } else { false })
                                        .unwrap()
                                        .0,
                                );
                                continue;
                            }
                            Some(ref mut j) => {
                                pool.1 -= pool.0[*j].as_ref().unwrap().capacity();

                                // We can use swap remove because we remove the largest (and last) buffer (that isn't None) from the vec.
                                pool.0.swap_remove(*j);
                                *j = j.wrapping_sub(1);
                                continue;
                            }
                        }
                    }
                    break;
                }
                return;
            }
        }
        #[cfg(debug)]
        unreachable!()
    })
}

#[repr(transparent)]
pub struct BufGuard(ManuallyDrop<Vec<u8>>);

impl BufGuard {
    fn new(v: Vec<u8>) -> BufGuard {
        Self(ManuallyDrop::new(v))
    }
}

impl Deref for BufGuard {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BufGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for BufGuard {
    fn drop(&mut self) {
        return_buf(unsafe { ManuallyDrop::take(&mut self.0) });
    }
}

#[cfg(test)]
#[test]
fn test() {
    use rand::{seq::SliceRandom, Rng};

    let mut rng = rand::thread_rng();
    let mut a = request_buf(32);
    a.push(1);
    a.push(2);
    let mut b = request_buf(64);
    drop(a);
    b.push(3);
    b.push(4);
    let mut bufs = Vec::new();
    for _ in 0..=16 {
        bufs.push(request_buf(64 * rng.gen_range(0..16)));
    }
    bufs.shuffle(&mut rng);
    for _ in 0..8 {
        bufs.pop();
    }
    for _ in 0..=16 {
        bufs.push(request_buf(64 * rng.gen_range(0..16)));
    }
}
