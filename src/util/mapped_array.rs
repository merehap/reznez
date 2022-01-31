use std::ops::{Index, IndexMut};

// One KibiByte.
const CHUNK_LEN: usize = 0x400;
const DUMMY_CHUNK: &'static [u8; CHUNK_LEN] = &[0; CHUNK_LEN];

pub struct MappedArray<'a, const CHUNK_COUNT: usize>(
    [&'a [u8; CHUNK_LEN]; CHUNK_COUNT]
);

impl <'a, const CHUNK_COUNT: usize> MappedArray<'a, CHUNK_COUNT> {
    // PERFORMANCE: Array allocation here is a bottleneck.
    // If there are no mappers that use under 8KiB PRG bank sizes,
    // then it probably makes sense to make CHUNK_LEN a const generic.
    // Some perf can be gained once consts can be used in macros:
    // https://github.com/rust-lang/rust/issues/52393
    pub fn new<const LEN: usize>(
        bytes: &'a [u8; LEN],
    ) -> MappedArray<'a, CHUNK_COUNT> {
        assert_eq!(LEN, CHUNK_COUNT * CHUNK_LEN,
            "LEN == CHUNK_COUNT * CHUNK_LEN must be true but {} != {} * {}",
            LEN, CHUNK_COUNT, CHUNK_LEN,
        );

        let mut chunks = [DUMMY_CHUNK; CHUNK_COUNT];

        let mut chunks_iter = bytes.array_chunks::<CHUNK_LEN>();
        for i in 0..chunks.len() {
            chunks[i] = chunks_iter.next().unwrap();
        }

        MappedArray(chunks)
    }

    pub fn from_halves<const LEN: usize>(
        first_half: &'a [u8; LEN],
        second_half: &'a [u8; LEN],
    ) -> MappedArray<'a, CHUNK_COUNT> {
        assert_eq!(2 * LEN, CHUNK_COUNT * CHUNK_LEN,
            "2 * LEN == CHUNK_COUNT * CHUNK_LEN must be true but {} != {} * {}",
            2 * LEN, CHUNK_COUNT, CHUNK_LEN,
        );

        let mut chunks = [DUMMY_CHUNK; CHUNK_COUNT];

        let mut index = 0;
        for chunk in first_half.array_chunks::<CHUNK_LEN>() {
            chunks[index] = chunk;
            index += 1;
        }

        for chunk in second_half.array_chunks::<CHUNK_LEN>() {
            chunks[index] = chunk;
            index += 1;
        }

        assert_eq!(index, chunks.len());

        MappedArray(chunks)
    }
}

impl <const CHUNK_COUNT: usize> Index<usize> for MappedArray<'_, CHUNK_COUNT> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx / CHUNK_LEN][idx % CHUNK_LEN]
    }
}

pub struct MappedArrayMut<'a, const CHUNK_COUNT: usize>(
    [&'a mut [u8; CHUNK_LEN]; CHUNK_COUNT]
);

impl <'a, const CHUNK_COUNT: usize> MappedArrayMut<'a, CHUNK_COUNT> {
    pub fn new<const LEN: usize>(
        bytes: &'a mut [u8; LEN],
    ) -> MappedArrayMut<'a, CHUNK_COUNT> {
        assert_eq!(LEN, CHUNK_COUNT * CHUNK_LEN,
            "LEN == CHUNK_COUNT * CHUNK_LEN was false. {} != {} * {}",
            LEN, CHUNK_COUNT, CHUNK_LEN,
        );

        let chunks: Vec<&mut [u8; CHUNK_LEN]> = bytes.array_chunks_mut().collect();
        MappedArrayMut(chunks.try_into().unwrap())
    }
}

impl <const CHUNK_COUNT: usize> Index<usize> for MappedArrayMut<'_, CHUNK_COUNT> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx / CHUNK_LEN][idx % CHUNK_LEN]
    }
}

impl <const CHUNK_COUNT: usize> IndexMut<usize> for MappedArrayMut<'_, CHUNK_COUNT> {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx / CHUNK_LEN][idx % CHUNK_LEN]
    }
}
