use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct ConstVec<T: Copy, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> ConstVec<T, N> {
    pub const fn new() -> Self {
        let data: MaybeUninit<[T; N]> = MaybeUninit::uninit();
        let data =
            unsafe { (&data as *const MaybeUninit<[T; N]> as *const [MaybeUninit<T>; N]).read() };
        Self { data, len: 0 }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn try_push(&mut self, value: T) -> Option<()> {
        if self.len >= N {
            return None;
        }
        self.data[self.len] = MaybeUninit::new(value);
        self.len += 1;
        Some(())
    }

    pub const fn push(&mut self, value: T) {
        self.try_push(value).expect("vector is full");
    }

    pub const fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let res = unsafe { *self.get_unchecked(self.len) };
        self.len -= 1;
        Some(res)
    }

    pub const unsafe fn get_unchecked(&self, index: usize) -> &T {
        unsafe { self.data[index].assume_init_ref() }
    }

    pub const fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(self.get_unchecked(index)) }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        (0..self.len).map(|i| unsafe { self.get_unchecked(i) })
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
    }
}

impl<T: Copy, const N: usize> Default for ConstVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> Clone for ConstVec<T, N> {
    fn clone(&self) -> Self {
        let mut new = ConstVec::new();
        for &elem in self.iter() {
            new.push(elem);
        }
        new
    }
}

impl<T: Copy, const N: usize> Copy for ConstVec<T, N> {}

impl<T: Copy, const N: usize> std::ops::Deref for ConstVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Copy, const N: usize> std::ops::DerefMut for ConstVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

#[macro_export]
macro_rules! const_vec {
    ($($item:expr),*$(,)?) => {
        const {
            let mut v = ConstVec::new();
            $(v.push($item);)*
            v
        }
    };
}