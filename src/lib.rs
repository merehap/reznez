#![feature(array_chunks)]
#![feature(slice_as_chunks)]
#![feature(const_option)]
#![feature(if_let_guard)]
#![feature(type_ascription)]
#![feature(let_chains)]
#![feature(const_option_ext)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

pub mod apu;
pub mod analysis;
pub mod cartridge;
pub mod config;
pub mod controller;
pub mod cpu;
pub mod gui;
pub mod memory;
pub mod nes;
pub mod ppu;
pub mod util;
