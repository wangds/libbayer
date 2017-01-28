//! This crate provides routines for demosaicing Bayer raw images.

extern crate byteorder;
extern crate libc;

#[macro_use]
extern crate quick_error;

pub use errcode::BayerError;
pub use errcode::BayerResult;
pub use raster::RasterDepth;

/// Mutable raster structure.
#[allow(dead_code)]
pub struct RasterMut<'a> {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    stride: usize,
    depth: RasterDepth,
    buf: &'a mut [u8],
}

pub mod ffi;

mod bayer;
mod border_none;
mod errcode;
mod raster;
