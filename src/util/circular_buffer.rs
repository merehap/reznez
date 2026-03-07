pub struct CircularBuffer<T> {
    // TODO: Replace this with a boxed array.
    buffer: Vec<T>,
    start_index: usize,
}

impl <T: Clone + Default> CircularBuffer<T> {
    pub fn default_filled(len: u32) -> Self {
        let len: usize = len.try_into().expect("Circular buffer len to fit in a usize.");
        Self {
            buffer: vec![T::default(); len],
            start_index: 0,
        }
    }
}

impl <T: Clone> CircularBuffer<T> {
    // Pushing always pushes to the 'last' element, even the very first push.
    pub fn push(&mut self, value: T) {
        self.buffer[self.start_index] = value;
        self.start_index = self.start_index.wrapping_add(1);
        self.start_index %= self.buffer.len();
    }

    pub fn clone_to(&self, target: &mut [[T; 2]]) {
        // TODO: Enforce this at compile time.
        assert_eq!(target.len(), self.buffer.len());

        let split_point = self.buffer.len().strict_sub(self.start_index);
        for i in 0..split_point {
            target[i][1] = self.buffer[self.start_index + i].clone();
        }

        for i in split_point..self.buffer.len() {
            target[i][1] = self.buffer[i - split_point].clone();
        }
    }
}