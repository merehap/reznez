use std::fs::OpenOptions;
use std::num::{NonZeroU16, NonZeroU8};
use std::ops::{Index, IndexMut, Range, RangeInclusive};
use std::path::Path;

use log::warn;
use memmap2::MmapMut;

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

    pub fn peek_u64(&self, range: RangeInclusive<u32>) -> Option<u64> {
        assert_eq!(range.end() - range.start(), 7);
        self.0.get(*range.start() as usize..=*range.end() as usize)
            .map(|slice| u64::from_be_bytes(slice.try_into().unwrap()))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }

    pub fn slice(&self, range: Range<u32>) -> RawMemorySlice<'_> {
        RawMemorySlice(&self.0[range.start as usize..range.end as usize])
    }

    pub fn maybe_slice(&self, range: Range<u32>) -> Option<RawMemorySlice<'_>> {
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

    pub fn split_n(self, count: NonZeroU8) -> Vec<RawMemory> {
        if self.0.is_empty() {
            return Vec::new();
        }

        let results: Vec<_> = self.0.chunks_exact(self.0.len() / usize::from(count.get()))
            .map(|chunk| RawMemory(chunk.to_vec()))
            .collect();
        assert_eq!(results.len(), usize::from(count.get()));
        results
    }

    pub fn chunks(self, size: NonZeroU16) -> Vec<RawMemory> {
        if self.0.is_empty() {
            return Vec::new()
        } else if self.0.len() < usize::from(size.get()) {
            return vec![self];
        }

        assert_eq!(self.0.len() % usize::from(size.get()), 0);
        self.0.chunks(size.get() as usize)
            .map(|chunk| RawMemory(chunk.to_vec()))
            .collect()
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

pub struct SaveRam {
    mode_state: SaveRamModeState,
}

impl SaveRam {
    pub fn empty() -> Self {
        SaveRam { mode_state: SaveRamModeState::NonSaving(vec![0; 0]) }
    }

    pub fn open(path: &Path, size: u32, allow_saving: bool) -> Self {
        if size == 0 {
            return SaveRam { mode_state: SaveRamModeState::Empty };
        }

        if !allow_saving {
            return SaveRam { mode_state: SaveRamModeState::NonSaving(vec![0; size as usize]) }
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path);
        let mode_state = file
            .and_then(|file| {
                file.set_len(size as u64)?;
                // SAFETY: Unsafe. We can't guarantee that another process doesn't modify the file.
                unsafe { MmapMut::map_mut(&file) }
            })
            .map_err(|err| warn!("Failed to load or create Save RAM at {}. RAM will be lost upon exit. {err}", path.display()))
            .map(SaveRamModeState::Saving)
            .unwrap_or(SaveRamModeState::NonSaving(vec![0; size as usize]));

        SaveRam { mode_state }
    }

    pub fn size(&self) -> u32 {
        match &self.mode_state {
            SaveRamModeState::Empty => 0,
            SaveRamModeState::NonSaving(vec) => vec.len() as u32,
            SaveRamModeState::Saving(mmap) => mmap.len() as u32,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.mode_state, SaveRamModeState::Empty)
    }
}

impl Index<u32> for SaveRam {
    type Output = u8;

    fn index(&self, index: u32) -> &u8 {
        match &self.mode_state {
            SaveRamModeState::Empty => panic!("Can't read from empty Save RAM."),
            SaveRamModeState::NonSaving(vec) => &vec[index as usize],
            SaveRamModeState::Saving(mmap) => &mmap[index as usize],
        }
    }
}

impl IndexMut<u32> for SaveRam {
    fn index_mut(&mut self, index: u32) -> &mut u8 {
        match &mut self.mode_state {
            SaveRamModeState::Empty => panic!("Can't read from empty Save RAM."),
            SaveRamModeState::NonSaving(vec) => &mut vec[index as usize],
            SaveRamModeState::Saving(mmap) => &mut mmap[index as usize],
        }
    }
}

enum SaveRamModeState {
    Empty,
    NonSaving(Vec<u8>),
    Saving(MmapMut),
}