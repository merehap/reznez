use std::cell::RefCell;
use std::rc::Rc;

// One KibiByte.
const CHUNK_LEN: usize = 0x400;

pub struct MappedArray<const CHUNK_COUNT: usize>([Chunk; CHUNK_COUNT]);

impl <const CHUNK_COUNT: usize> MappedArray<CHUNK_COUNT> {
    pub fn empty() -> MappedArray<CHUNK_COUNT> {
        let mut chunks = Vec::new();
        for _ in 0..CHUNK_COUNT {
            chunks.push(Chunk::empty());
        }

        MappedArray(chunks.try_into().unwrap())
    }

    pub fn new<const LEN: usize>(backing: [u8; LEN]) -> MappedArray<CHUNK_COUNT> {
        let mut array = MappedArray::empty();
        array.update(Rc::new(RefCell::new(backing)));
        array
    }

    pub fn mirror_half<const LEN: usize>(half: [u8; LEN]) -> MappedArray<32> {
        let half = Rc::new(RefCell::new(half));
        let mut array = MappedArray::empty();
        array.update_from_halves(half.clone(), half);
        array
    }

    pub fn from_halves<const LEN: usize>(
        first_half: [u8; LEN],
        second_half: [u8; LEN],
    ) -> MappedArray<32> {
        let mut array = MappedArray::empty();
        array.update_from_halves(
            Rc::new(RefCell::new(first_half)),
            Rc::new(RefCell::new(second_half)),
        );
        array
    }

    pub fn update<const LEN: usize>(&mut self, backing: Rc<RefCell<[u8; LEN]>>) {
        assert_eq!(LEN, CHUNK_COUNT * CHUNK_LEN,
            "LEN == CHUNK_COUNT * CHUNK_LEN must be true but {} != {} * {}",
            LEN, CHUNK_COUNT, CHUNK_LEN,
        );

        for (i, chunk) in self.0.iter_mut().enumerate() {
            *chunk = Chunk::new(backing.clone(), i * CHUNK_LEN);
        }
    }

    pub fn update_from_halves<const LEN: usize>(
        &mut self,
        first_half: Rc<RefCell<[u8; LEN]>>,
        second_half: Rc<RefCell<[u8; LEN]>>,
    ) {
        assert_eq!(2 * LEN, CHUNK_COUNT * CHUNK_LEN,
            "2 * LEN == CHUNK_COUNT * CHUNK_LEN must be true but {} != {} * {}",
            2 * LEN, CHUNK_COUNT, CHUNK_LEN,
        );

        let half_count = CHUNK_COUNT / 2;
        for i in 0..half_count {
            self.0[i] = Chunk::new(first_half.clone(), i * CHUNK_LEN);
        }

        for i in 0..half_count {
            self.0[i + half_count] = Chunk::new(second_half.clone(), i * CHUNK_LEN);
        }
    }

    pub fn read(&self, index: usize) -> u8 {
        self.0[index / CHUNK_LEN].read(index % CHUNK_LEN)
    }

    pub fn write(&self, index: usize, value: u8) {
        self.0[index / CHUNK_LEN].write(index % CHUNK_LEN, value);
    }
}

#[derive(Clone, Debug)]
struct Chunk {
    backing: Rc<RefCell<[u8]>>,
    start_index: usize,
}

impl Chunk {
    pub fn empty() -> Chunk {
        Chunk {
            backing: Rc::new(RefCell::new([0; CHUNK_LEN])),
            start_index: 0,
        }
    }

    pub fn new(backing: Rc<RefCell<[u8]>>, start_index: usize) -> Chunk {
        assert!(backing.borrow().len() >= CHUNK_LEN);

        Chunk {backing, start_index}
    }

    fn read(&self, index: usize) -> u8 {
        self.backing.borrow()[self.start_index + index]
    }

    fn write(&self, index: usize, value: u8) {
        self.backing.borrow_mut()[self.start_index + index] = value;
    }
}
