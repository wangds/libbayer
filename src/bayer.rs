//! Bayer image definitions.

use std::io::Read;
use byteorder::{BigEndian,LittleEndian,ReadBytesExt};

use ::BayerResult;

/// The 2x2 colour filter array (CFA) pattern.
///
/// The sequence of R, G, B describe the colours of the top-left,
/// top-right, bottom-left, and bottom-right pixels in the 2x2 block,
/// in that order.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum CFA {
    BGGR,
    GBRG,
    GRBG,
    RGGB,
}

/// The depth and endianness of the raw image.
///
/// Note that many cameras only capture 12-bits per pixel, but still
/// store the data as 16-bits per pixel.  These should be treated as
/// 16-bits per pixel for the purposes of this library.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum BayerDepth {
    Depth8,
    Depth16BE,
    Depth16LE,
}

/// Trait for reading 8-bpp Bayer lines.
pub trait BayerRead8 {
    fn read_line(&self, r: &mut Read, dst: &mut [u8]) -> BayerResult<()>;
}

/// Trait for reading 16-bpp Bayer lines, big-endian or little-endian.
pub trait BayerRead16 {
    fn read_line(&self, r: &mut Read, dst: &mut [u16]) -> BayerResult<()>;
}

/// Read the exact number of bytes required to fill buf.
/// For u8 source data.
pub fn read_exact_u8(r: &mut Read, buf: &mut [u8])
        -> BayerResult<()> {
    r.read_exact(buf)?;
    Ok(())
}

/// Read the exact number of bytes required to fill buf.
/// For u16 big-endian source data.
pub fn read_exact_u16be(r: &mut Read, buf: &mut [u16])
        -> BayerResult<()> {
    for i in 0..buf.len() {
        buf[i] = r.read_u16::<BigEndian>()?;
    }
    Ok(())
}

/// Read the exact number of bytes required to fill buf.
/// For u16 little-endian source data.
pub fn read_exact_u16le(r: &mut Read, buf: &mut [u16])
        -> BayerResult<()> {
    for i in 0..buf.len() {
        buf[i] = r.read_u16::<LittleEndian>()?;
    }
    Ok(())
}

impl CFA {
    /// The 2x2 pixel block obtained when moving right 1 column.
    pub fn next_x(self) -> Self {
        match self {
            CFA::BGGR => CFA::GBRG,
            CFA::GBRG => CFA::BGGR,
            CFA::GRBG => CFA::RGGB,
            CFA::RGGB => CFA::GRBG,
        }
    }

    /// The 2x2 pixel block obtained when moving down 1 row.
    pub fn next_y(self) -> Self {
        match self {
            CFA::BGGR => CFA::GRBG,
            CFA::GBRG => CFA::RGGB,
            CFA::GRBG => CFA::BGGR,
            CFA::RGGB => CFA::GBRG,
        }
    }
}
