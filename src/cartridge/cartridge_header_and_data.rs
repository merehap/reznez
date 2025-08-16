#![allow(dead_code)]
#![allow(unused_variables)]

use std::path::Path;

use crate::memory::raw_memory::RawMemory;

pub struct CartridgeHeaderAndData {
    header_and_data: RawMemory,
}

impl CartridgeHeaderAndData {
    pub fn new(raw_header_and_data: RawMemory, path: &Path, allow_saving: bool) -> Result<Self, String> {
        panic!();
    }
}