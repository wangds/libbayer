//! This crate provides routines for demosaicing Bayer raw images.

extern crate byteorder;
extern crate libc;

#[macro_use]
extern crate quick_error;

pub use errcode::BayerError;
pub use errcode::BayerResult;

mod errcode;
