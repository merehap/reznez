#![feature(array_chunks)]
#![feature(const_option)]
#![feature(destructuring_assignment)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]

pub mod cartridge;
pub mod config;
pub mod cpu;
mod ppu;
mod mapper;
pub mod nes;
mod util;
