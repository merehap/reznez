use std::ops::{Index, IndexMut};

// One KibiByte.
const CHUNK_LEN: usize = 0x400;

pub struct MappedArray<'a, const CHUNK_COUNT: usize>(
    [&'a [u8; CHUNK_LEN]; CHUNK_COUNT]
);

impl <'a, const CHUNK_COUNT: usize> MappedArray<'a, CHUNK_COUNT> {
    pub fn new<const LEN: usize>(
        bytes: &'a [u8; LEN],
    ) -> MappedArray<'a, CHUNK_COUNT> {
        assert_eq!(LEN, CHUNK_COUNT * CHUNK_LEN,
            "LEN == CHUNK_COUNT * CHUNK_LEN was false. {} != {} * {}",
            LEN, CHUNK_COUNT, CHUNK_LEN,
        );

        println!("LEN: {} ({}), CHUNK_COUNT: {}", LEN, bytes.len(), CHUNK_COUNT);
        let chunks: Vec<&[u8; CHUNK_LEN]> = bytes.array_chunks::<CHUNK_LEN>().collect();
        println!("ACTUAL CHUNK COUNT: {}. LEN: {}", chunks.len(), chunks[0].len());
        MappedArray(chunks.try_into().unwrap())
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
