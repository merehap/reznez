#![feature(array_chunks)]
#![feature(slice_as_chunks)]
#![feature(const_option)]
#![feature(if_let_guard)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

pub mod cartridge;
pub mod config;
pub mod controller;
pub mod cpu;
pub mod gui;
pub mod ppu;
pub mod memory;
pub mod nes;
pub mod util;
