//! Bayer image definitions.

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::Read;

use crate::BayerResult;

/// The 2×2 colour filter array (CFA) pattern.
///
/// The sequence of R, G, B describe the colours of the top-left,
/// top-right, bottom-left, and bottom-right pixels in the 2×2 block,
/// in that order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CFA {
    BGGR,
    GBRG,
    GRBG,
    RGGB,
}

/// The depth and endianness of the raw image.
///
/// Note that many cameras only capture 12 bits per pixel, but still
/// store the data as 16-bits per pixel.  These should be treated as
/// 16 bits per pixel for the purposes of this library.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BayerDepth {
    Depth8,
    Depth16BE,
    Depth16LE,
}

/// Trait for reading 8 bit per pixel Bayer lines.
pub trait BayerRead8 {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u8]) -> BayerResult<()>;
}

/// Trait for reading 16 bit per pixel Bayer lines. Big-endian or little-endian.
pub trait BayerRead16 {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u16]) -> BayerResult<()>;
}

/// Read the exact number of bytes required to fill `buf`.
/// For [`u8`] source data.
pub fn read_exact_u8(r: &mut dyn Read, buf: &mut [u8]) -> BayerResult<()> {
    r.read_exact(buf)?;
    Ok(())
}

/// Read the exact number of bytes required to fill `buf`.
/// For [`u16`] big-endian source data.
pub fn read_exact_u16be(r: &mut dyn Read, buf: &mut [u16]) -> BayerResult<()> {
    for item in buf {
        *item = r.read_u16::<BigEndian>()?;
    }
    Ok(())
}

/// Read the exact number of bytes required to fill `buf`.
/// For [`u16`] little-endian source data.
pub fn read_exact_u16le(r: &mut dyn Read, buf: &mut [u16]) -> BayerResult<()> {
    for item in buf {
        *item = r.read_u16::<LittleEndian>()?;
    }
    Ok(())
}

impl CFA {
    /// The 2×2 pixel block obtained when moving right one column.
    pub fn next_x(self) -> Self {
        match self {
            CFA::BGGR => CFA::GBRG,
            CFA::GBRG => CFA::BGGR,
            CFA::GRBG => CFA::RGGB,
            CFA::RGGB => CFA::GRBG,
        }
    }

    /// The 2×2 pixel block obtained when moving down one row.
    pub fn next_y(self) -> Self {
        match self {
            CFA::BGGR => CFA::GRBG,
            CFA::GBRG => CFA::RGGB,
            CFA::GRBG => CFA::BGGR,
            CFA::RGGB => CFA::GBRG,
        }
    }
}
