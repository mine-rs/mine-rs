use std::{
    cell::Cell,
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
    static BUFPOOL: Cell<(Vec<Option<Vec<u8>>>, usize)> = Cell::new((vec![None], 0));
}

pub fn request_largest_buf() -> BufGuard {
    let mut pool = BUFPOOL.take();
    for v in pool.0.iter_mut().rev() {
        match v {
            Some(u) => {
                pool.1 -= u.capacity();
                let buf = BufGuard::new(std::mem::take(v).unwrap());
                BUFPOOL.set(pool);
                return buf;
            }
            None => continue,
        }
    }
    // No buffers were available.
    pool.0.push(None);
    BUFPOOL.set(pool);
    BufGuard::new(Vec::with_capacity(8 * 1024))
}

pub fn request_buf(req_cap: usize) -> BufGuard {
    let mut pool = BUFPOOL.take();
    let last = pool.0.len() - 1;
    for (i, v) in pool.0.iter_mut().enumerate() {
        match v {
            Some(u) => {
                if u.capacity() >= req_cap {
                    pool.1 -= u.capacity();
                    let buf = BufGuard::new(std::mem::take(v).unwrap());
                    BUFPOOL.set(pool);
                    return buf;
                } else if i == last {
                    u.reserve(req_cap - u.capacity());
                    pool.1 -= u.capacity();
                    let buf = BufGuard::new(std::mem::take(v).unwrap());
                    BUFPOOL.set(pool);
                    return buf;
                }
            }
            None => continue,
        }
    }
    // No buffers were available.
    pool.0.push(None);
    BUFPOOL.set(pool);
    BufGuard::new(Vec::with_capacity(req_cap))
}

fn reorder_pool() {
    let mut pool = BUFPOOL.take();
    pool.0.sort_by(|a, b| {
        // Order the vec from least to most capacity.
        match (a, b) {
            (None, None) | (None, _) | (_, None) => Ordering::Equal,
            (Some(a), Some(b)) => a.capacity().cmp(&b.capacity()),
        }
    });
    BUFPOOL.set(pool)
}

/// Cleans up some memory if usage exceeds maximum.
fn cleanup(pool: &mut (Vec<Option<Vec<u8>>>, usize)) {
    let mut j: Option<usize> = None;
    loop {
        if !(&pool.1 > &BUFPOOL_MAX_MEMORY) {
            break;
        }
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
}

fn return_buf(buf: Vec<u8>) {
    let mut pool = BUFPOOL.take();
    pool.1 += buf.capacity();
    for i in &mut pool.0 {
        if i.is_none() {
            *i = Some(buf);
            reorder_pool();
            cleanup(&mut pool);
            BUFPOOL.set(pool);
            return;
        }
    }
    // To prevent a crash, just in case.
    BUFPOOL.set(pool);
    #[cfg(debug)]
    unreachable!();
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
