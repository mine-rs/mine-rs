use std::{cell::UnsafeCell, ops::{Deref, DerefMut}, cmp::Ordering, mem::ManuallyDrop};

thread_local! {
    static BUF_POOL: UnsafeCell<Vec<Option<Vec<u8>>>> = UnsafeCell::new(vec![None]);
}

pub fn request_buf(req_cap: usize) -> BufGuard {
    BufGuard::new(BUF_POOL.with(|x| {
        let x =  unsafe {x.get().as_mut().unwrap() };
        let last = x.len() - 1;
        for (i, v) in x.iter_mut().enumerate() {
            match v {
                Some(v) => {
                    if v.capacity() >= req_cap {
                    } else if i == last {
                        v.reserve(req_cap - v.capacity());
                        return std::mem::take(v);
                    }
                },
                None => continue,
            }
        }
        // No buffers were available.
        x.push(None);
        Vec::with_capacity(req_cap)
    }))
}

fn reorder_pool() {
    BUF_POOL.with(|x| {
        let x =  unsafe {x.get().as_mut().unwrap() };
        x.sort_by(|a, b| {
            // Order the vec from least to most capacity, with the taken buffers being put at the end of the vec.
            if let None = a {
                Ordering::Greater
            } else if let None = b {
                Ordering::Less
            } else {
                a.as_ref().unwrap().capacity().cmp(&b.as_ref().unwrap().capacity())
            }

        } )
    });
}

fn return_buf(buf: Vec<u8>) {
    BUF_POOL.with(|x| {
        let x =  unsafe {x.get().as_mut().unwrap() };
        for i in x {
            if i.is_none() {
                *i = Some(buf);
                reorder_pool();
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
    use rand::{Rng, seq::SliceRandom};

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
