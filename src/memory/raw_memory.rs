use std::ops::{Index, IndexMut, Range};

// A chunk of primitive memory. Allows indexing on u32s instead of usizes.
#[derive(Clone, Debug)]
pub struct RawMemory(Vec<u8>);

impl RawMemory {
    pub fn new(size: u32) -> Self {
        Self(vec![0; size as usize])
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        assert!(vec.len() <= u32::MAX.try_into().unwrap());
        Self(vec)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }

    pub fn slice(&self, range: Range<u32>) -> RawMemorySlice {
        RawMemorySlice(&self.0[range.start as usize..range.end as usize])
    }

    pub fn maybe_slice(&self, range: Range<u32>) -> Option<RawMemorySlice> {
        self.0.get(range.start as usize..range.end as usize)
            .map(RawMemorySlice)
    }

    pub fn sized_slice<const SIZE: usize>(&self, start: u32) -> &[u8; SIZE] {
        let start = start as usize;
        (&self.0[start..start + SIZE]).try_into().unwrap()
    }

    pub fn sized_slice_mut<const SIZE: usize>(&mut self, start: u32) -> &mut [u8; SIZE] {
        let start = start as usize;
        (&mut self.0[start..start + SIZE]).try_into().unwrap()
    }

    pub fn split_n(self, count: u8) -> Vec<RawMemory> {
        let results: Vec<_> = self.0.chunks_exact(self.0.len() / usize::from(count))
            .map(|chunk| RawMemory(chunk.to_vec()))
            .collect();
        assert_eq!(results.len(), usize::from(count));
        results
    }

    pub fn size(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Index<u32> for RawMemory {
    type Output = u8;

    fn index(&self, index: u32) -> &u8 {
        &self.0[index as usize]
    }
}

impl IndexMut<u32> for RawMemory {
    fn index_mut(&mut self, index: u32) -> &mut u8 {
        &mut self.0[index as usize]
    }
}

// A chunk of primitive memory with a known size at compile time.
// Allows indexing on u32s instead of usizes.
//
// An array is not the inner type because:
// * We need a u32 for size, but an array can't be indexed by a u32, and const generics doesn't
// allow 'SIZE as usize' in the type position yet.
// * Arrays require stack allocation and cause stack overflows.
#[derive(Clone, Debug)]
pub struct RawMemoryArray<const SIZE: u32>(Box<[u8]>);

impl <const SIZE: u32> RawMemoryArray<SIZE> {
    pub fn new() -> Self {
        RawMemoryArray(vec![0; SIZE as usize].into_boxed_slice())
    }

    pub fn as_raw_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn as_raw_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl <const SIZE: u32> Index<u32> for RawMemoryArray<SIZE> {
    type Output = u8;

    fn index(&self, index: u32) -> &u8 {
        &self.0[index as usize]
    }
}

impl <const SIZE: u32> IndexMut<u32> for RawMemoryArray<SIZE> {
    fn index_mut(&mut self, index: u32) -> &mut u8 {
        &mut self.0[index as usize]
    }
}

#[derive(Clone, Debug)]
pub struct RawMemorySlice<'a>(&'a [u8]);

impl<'a> RawMemorySlice<'a> {
    pub fn from_raw(raw: &'a [u8]) -> RawMemorySlice<'a> {
        RawMemorySlice(raw)
    }

    pub fn to_raw(&'a self) -> &'a [u8] {
        self.0
    }

    pub fn to_raw_memory(&self) -> RawMemory {
        RawMemory::from_vec(self.0.to_vec())
    }

    pub fn size(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Index<u32> for RawMemorySlice<'_> {
    type Output = u8;

    fn index(&self, index: u32) -> &u8 {
        &self.0[index as usize]
    }
}
