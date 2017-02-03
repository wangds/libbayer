//! This crate provides routines for demosaicing Bayer raw images.

extern crate byteorder;
extern crate libc;

#[macro_use]
extern crate quick_error;

use std::io::Read;

pub use bayer::BayerDepth;
pub use bayer::CFA;
pub use demosaic::Demosaic;
pub use errcode::BayerError;
pub use errcode::BayerResult;
pub use raster::RasterDepth;

/// Mutable raster structure.
pub struct RasterMut<'a> {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    stride: usize,
    depth: RasterDepth,
    buf: &'a mut [u8],
}

pub mod demosaic;
pub mod ffi;

mod bayer;
mod border_mirror;
mod border_none;
mod border_replicate;
mod errcode;
mod raster;

/// Run the demosaicing algorithm on the Bayer image.
///
/// # Example
///
/// ```
/// use std::io::Cursor;
///
/// let width: usize = 320;
/// let height: usize = 200;
/// let img = vec![0; width * height];
/// let mut buf = vec![0; 3 * width * height];
///
/// let mut dst = bayer::RasterMut::new(
///         width, height, bayer::RasterDepth::Depth8,
///         &mut buf);
/// bayer::run_demosaic(&mut Cursor::new(&img[..]),
///         bayer::BayerDepth::Depth8,
///         bayer::CFA::RGGB,
///         bayer::Demosaic::None,
///         &mut dst);
/// ```
pub fn run_demosaic(r: &mut Read,
        depth: BayerDepth, cfa: CFA, alg: Demosaic,
        dst: &mut RasterMut)
        -> BayerResult<()> {
    match alg {
        Demosaic::None => demosaic::none::run(r, depth, cfa, dst),
        Demosaic::NearestNeighbour => demosaic::nearestneighbour::run(r, depth, cfa, dst),
        Demosaic::Linear => demosaic::linear::run(r, depth, cfa, dst),
        Demosaic::Cubic => demosaic::cubic::run(r, depth, cfa, dst),
    }
}
