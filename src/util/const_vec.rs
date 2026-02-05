use std::mem::MaybeUninit;

// A Vec-like collection with only const creation functions and methods.
#[derive(Clone, Copy, Debug)]
pub struct ConstVec<T: Clone + Copy, const CAPACITY: usize> {
    backing: [MaybeUninit<T>; CAPACITY],
    len: u8,
}

impl <T: Clone + Copy, const CAPACITY: usize> ConstVec<T, CAPACITY> {
    pub const fn new() -> ConstVec<T, CAPACITY> {
        ConstVec {
            backing: [const { MaybeUninit::uninit() }; CAPACITY],
            len: 0,
        }
    }

    pub const fn len(&self) -> u8 {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn get(&self, index: u8) -> T {
        assert!(index < self.len);
        // SAFETY: The assertion above ensures that the index is initialized.
        unsafe { self.backing[index as usize].assume_init() }
    }

    pub const fn get_ref(&self, index: u8) -> &T {
        assert!(index < self.len);
        // SAFETY: The assertion above ensures that the index is initialized.
        unsafe { self.backing[index as usize].assume_init_ref() }
    }

    pub const fn get_mut(&mut self, index: u8) -> &mut T {
        assert!(index < self.len);
        // SAFETY: The assertion above ensures that the index is initialized.
        unsafe { self.backing[index as usize].assume_init_mut() }
    }

    pub const fn push(&mut self, item: T) {
        self.len = self.len.checked_add(1)
            .expect("not more than 256 items to be pushed");
        assert!((self.len as usize) <= CAPACITY);

        self.backing[self.len as usize - 1].write(item);
    }

    pub fn as_iter(self) -> impl DoubleEndedIterator<Item = T> {
        self.backing.into_iter()
            .take(self.len as usize)
            // SAFETY: Values before the index have already been set.
            // TODO: Remove unsafe by implementing Default or similar.
            .map(|value| unsafe { value.assume_init() })
    }
}

impl <T: PartialEq + Clone + Copy, const CAPACITY: usize> PartialEq for ConstVec<T, CAPACITY> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        for (left, right) in self.backing.iter().zip(&other.backing).take(self.len as usize) {
            // SAFETY: We only "take" the initialized elements on the line above.
            if unsafe { left.assume_init() != right.assume_init() } {
                return false;
            }
        }

        true
    }
}

impl <T: PartialEq + Eq + Clone + Copy, const CAPACITY: usize> Eq for ConstVec<T, CAPACITY> {}