use std::mem::MaybeUninit;

// A Vec-like collection with only const creation functions and methods.
#[derive(Clone, Copy)]
pub struct ConstVec<T: Clone + Copy, const CAPACITY: usize> {
    backing: [MaybeUninit<T>; CAPACITY],
    index: u8,
}

impl <T: Clone + Copy, const CAPACITY: usize> ConstVec<T, CAPACITY> {
    pub const fn new() -> ConstVec<T, CAPACITY> {
        ConstVec {
            backing: [const { MaybeUninit::uninit() }; CAPACITY],
            index: 0,
        }
    }

    pub const fn push(&mut self, item: T) {
        self.index = self.index.checked_add(1)
            .expect("not more than 256 items to be pushed");
        assert!((self.index as usize) <= CAPACITY);

        self.backing[self.index as usize - 1].write(item);
    }

    pub const fn is_empty(&self) -> bool {
        self.index == 0
    }

    pub fn as_iter(self) -> impl Iterator<Item = T> {
        self.backing.into_iter()
            .take(self.index as usize)
            // SAFETY: Values before the index have already been set.
            // TODO: Remove unsafe by implementing Default or similar.
            .map(|value| unsafe { value.assume_init() })
    }
}
