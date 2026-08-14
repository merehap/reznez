#![feature(const_cmp)]
#![feature(const_index)]
#![feature(const_option_ops)]
#![feature(const_trait_impl)]
#![feature(const_try)]
#![feature(generic_const_parameter_types)]
#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![feature(map_try_insert)]
#![feature(variant_count)]
#![expect(incomplete_features)]
#![expect(clippy::module_inception)]
#![expect(clippy::new_without_default)]
#![expect(clippy::identity_op)]

pub mod apu;
pub mod analysis;
pub mod assembler;
pub mod bus;
pub mod cartridge;
pub mod circuit;
pub mod config;
pub mod controller;
pub mod counter;
pub mod cpu;
pub mod gui;
pub mod logging;
pub mod mapper;
pub mod master_clock;
pub mod memory;
pub mod nes;
pub mod ppu;
pub mod signals;
pub mod util;
