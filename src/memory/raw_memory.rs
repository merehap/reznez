use std::fs::OpenOptions;
use std::ops::{Index, IndexMut, Range, RangeInclusive};
use std::path::Path;

use log::warn;
use memmap2::MmapMut;

use crate::mapper::KIBIBYTE;

// A chunk of primitive memory. Allows indexing on u32s instead of usizes.
#[derive(Clone, Debug)]
pub struct RawData(Vec<u8>);

impl RawData {
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

    pub fn slice(&self, range: Range<u32>) -> RawMemorySlice<'_> {
        RawMemorySlice(&self.0[range.start as usize..range.end as usize])
    }

    pub fn maybe_slice(&self, range: Range<u32>) -> Option<RawMemorySlice<'_>> {
        self.0.get(range.start as usize..range.end as usize)
            .map(RawMemorySlice)
    }

    pub fn size(&self) -> u32 {
        self.0.len() as u32
    }
}

impl Index<u32> for RawData {
    type Output = u8;

    fn index(&self, index: u32) -> &u8 {
        &self.0[index as usize]
    }
}

impl IndexMut<u32> for RawData {
    fn index_mut(&mut self, index: u32) -> &mut u8 {
        &mut self.0[index as usize]
    }
}

pub enum PowerOf2VecResult {
    Absent,
    Exact(PowerOf2Vec),
    Split { start: PowerOf2Vec, remainder: Vec<u8> },
}

// A chunk of primitive memory. Allows indexing on u32s instead of usizes.
#[derive(Clone, Debug)]
pub enum RawMemory {
    Absent,
    OneChip(PowerOf2Vec),
    // The second chip will always be smaller than the first chip.
    TwoChips(PowerOf2Vec, PowerOf2Vec),
}

impl RawMemory {
    pub fn new(size: u32) -> Self {
        Self::from_vec(vec![0; size as usize])
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        assert!(vec.len() <= u32::MAX.try_into().unwrap());
        match PowerOf2Vec::new(vec) {
            PowerOf2VecResult::Absent => {
                RawMemory::Absent
            }
            PowerOf2VecResult::Exact(first_chip) => {
                RawMemory::OneChip(first_chip)
            }
            PowerOf2VecResult::Split { start: first_chip, remainder } => {
                let second_chip = PowerOf2Vec::strict(remainder);
                RawMemory::TwoChips(first_chip, second_chip)
            }
        }
    }

    /*
    pub fn mirror_until_power_of_two(self) -> Self {
        let len = self.0.len();
        if len.is_power_of_two() || len == 0 {
            // We're already at a target length, no need for modification.
            return self;
        }

        let magnitude = len.ilog2();
        let mirror_start = 2usize.pow(magnitude);
        let mirror_atom_size = len - mirror_start;
        assert!(mirror_start.is_multiple_of(mirror_atom_size), "Very weird mem size can't be mirrored properly.");
        let (multiple, remainder) = mirror_start.div_rem_euclid(&mirror_atom_size);
        assert_eq!(remainder, 0);

        let mut result = self.0.clone();
        for _ in 1..multiple {
            let mut next = self.0[mirror_start..].to_vec();
            result.append(&mut next);
        }

        assert_eq!(result.len(), 2 * mirror_start);

        Self(result)
    }
    */

    pub fn sized_slice<const SIZE: usize>(&self, start: u32) -> &[u8; SIZE] {
        assert_eq!(start & (KIBIBYTE - 1), 0);
        let start = start as usize;
        match self {
            Self::Absent => panic!("Can't take a slice of absent memory."),
            Self::OneChip(chip) => (&chip.0[start..start + SIZE]).try_into().unwrap(),
            Self::TwoChips(_first, _second) => todo!(),
        }
    }

    pub fn sized_slice_mut<const SIZE: usize>(&mut self, start: u32) -> &mut [u8; SIZE] {
        assert_eq!(start & (KIBIBYTE - 1), 0);
        let start = start as usize;
        match self {
            Self::Absent => panic!("Can't take a slice of absent memory."),
            Self::OneChip(chip) => (&mut chip.0[start..start + SIZE]).try_into().unwrap(),
            Self::TwoChips(_first, _second) => todo!(),
        }
    }

    pub fn hash(&self) -> u32 {
        let mut h = crc32fast::Hasher::new();
        match self {
            Self::Absent => { /* Nothing to do. */ }
            Self::OneChip(chip) => {
                h.update(&chip.0);
            }
            Self::TwoChips(first, second) => {
                h.update(&first.0);
                h.update(&second.0);
            }
        }

        h.finalize()
    }

    /*
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
    */

    /*
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
    */

    pub fn size(&self) -> u32 {
        match self {
            Self::Absent => 0,
            Self::OneChip(chip) => chip.len(),
            Self::TwoChips(first, second) => first.len().strict_add(second.len()),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Absent)
    }

    /*
    fn normalize_index(&self, index: u32) -> u32 {
        let size = self.size();
        if index < size {
            index
        } else if size.is_power_of_two() {
            index & (size - 1)
        } else if size == 0 {
            0
        } else {
            let second_chip_start = 1 << size.ilog2();
            let combined_chip_mask = (second_chip_start << 1) - 1;
            let index = index & combined_chip_mask;

            let second_chip_mask = size - second_chip_start;
            assert!(second_chip_mask.is_power_of_two(), "Bad memory size: 0b{size:b}");
            let remainder = index - second_chip_start;
            second_chip_start +
        }
    }
    */
}

impl Index<u32> for RawMemory {
    type Output = u8;

    fn index(&self, index: u32) -> &u8 {
        match self {
            Self::Absent => panic!("Can't index into Absent RawMemory."),
            Self::OneChip(chip) => &chip[index],
            Self::TwoChips(first, second) => {
                if index < first.len() {
                    &first[index]
                } else {
                    &second[index - first.len()]
                }
            }
        }
    }
}

impl IndexMut<u32> for RawMemory {
    fn index_mut(&mut self, index: u32) -> &mut u8 {
        match self {
            Self::Absent => panic!("Can't index into Absent RawMemory."),
            Self::OneChip(chip) => &mut chip[index],
            Self::TwoChips(first, second) => {
                if index < first.len() {
                    &mut first[index]
                } else {
                    &mut second[index - first.len()]
                }
            }
        }
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

#[derive(Clone, Debug)]
pub struct PowerOf2Vec(Vec<u8>);

impl PowerOf2Vec {
    pub fn strict(raw: Vec<u8>) -> Self {
        assert!(raw.len().is_power_of_two());
        Self(raw)
    }

    pub fn new(mut raw: Vec<u8>) -> PowerOf2VecResult {
        if raw.is_empty() {
            PowerOf2VecResult::Absent
        } else if raw.len().is_power_of_two() {
            PowerOf2VecResult::Exact(Self::strict(raw))
        } else {
            let split_point = 1 << raw.len().ilog2();
            let remainder = raw.split_off(split_point);
            PowerOf2VecResult::Split {
                start: Self::strict(raw),
                remainder,
            }
        }
    }

    pub fn len(&self) -> u32 {
        self.0.len().try_into().unwrap()
    }
}

impl Index<u32> for PowerOf2Vec {
    type Output = u8;

    fn index(&self, index: u32) -> &u8 {
        &self.0[index as usize]
    }
}

impl IndexMut<u32> for PowerOf2Vec {
    fn index_mut(&mut self, index: u32) -> &mut u8 {
        &mut self.0[index as usize]
    }
}