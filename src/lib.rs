//! Routines for demosaicing Bayer sensor (RAW) images.
//!
//! Both 8bit and 16bit images are supported.
//!
//! Several demosaicing algorithms are available. Pixels on the border
//! of the image are retained by replicating or mirroring the data in
//! the neighbourhood.
//!
//! ## Examples
//!
//! Open a Bayer file from disk:
//!
//! ```ignore
//! let mut file = File::open(Path::new("example.raw"))?;
//! ```
//!
//! This RAW data contains the red, green, or blue values at each pixel
//! only. There is no additional header data, so the image width and
//! height, pixel depth, and CFA pattern must be provided from elsewhere.
//!
//! Allocate the buffer to which we will decode the image:
//!
//! ```ignore
//! let img_w = 320;
//! let img_h = 200;
//! let depth = bayer::RasterDepth::Depth8;
//! let bytes_per_pixel = 3;
//! let mut buf = vec![0; bytes_per_pixel * img_w * img_h];
//!
//! let mut dst = bayer::RasterMut::new(img_w, img_h, depth, &mut buf);
//! ```
//!
//! Run the demosaicing process:
//!
//! ```ignore
//! let cfa = bayer::CFA::RGGB;
//! let alg = bayer::Demosaic::Linear;
//!
//! bayer::run_demosaic(&mut file, bayer::BayerDepth:Depth8, cfa, alg, &mut dst);
//! ```
//!
//! Note that many cameras will capture 12bits per pixel (channel), but
//! store the data as 16 bits per pixel.  These should be treated as
//! 16-bits per pixel for the purposes of this library.
pub use crate::{
    bayer::{BayerDepth, CFA},
    demosaic::Demosaic,
    errcode::{BayerError, BayerResult},
    raster::RasterDepth,
};
use std::io::Read;

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
/// let mut dst = bayer::RasterMut::new(width, height, bayer::RasterDepth::Depth8, &mut buf);
///
/// bayer::demosaic(
///     &mut Cursor::new(&img[..]),
///     bayer::BayerDepth::Depth8,
///     bayer::CFA::RGGB,
///     bayer::Demosaic::None,
///     &mut dst,
/// );
/// ```
pub fn demosaic(
    r: &mut dyn Read,
    depth: BayerDepth,
    cfa: CFA,
    alg: Demosaic,
    dst: &mut RasterMut,
) -> BayerResult<()> {
    match alg {
        Demosaic::None => demosaic::none::run(r, depth, cfa, dst),
        Demosaic::NearestNeighbour => demosaic::nearestneighbour::run(r, depth, cfa, dst),
        Demosaic::Linear => demosaic::linear::run(r, depth, cfa, dst),
        Demosaic::Cubic => demosaic::cubic::run(r, depth, cfa, dst),
    }
}
