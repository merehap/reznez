#![feature(if_let_guard)]
#![feature(type_ascription)]
#![feature(const_for)]
#![feature(const_option_ops)]
#![feature(const_trait_impl)]
#![feature(generic_const_parameter_types)]
#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![allow(incomplete_features)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]
#![allow(clippy::identity_op)]

pub mod apu;
pub mod analysis;
pub mod cartridge;
pub mod config;
pub mod controller;
pub mod cpu;
pub mod gui;
pub mod logging;
pub mod mapper;
pub mod mapper_list;
pub mod mappers;
pub mod memory;
pub mod nes;
pub mod ppu;
pub mod util;
