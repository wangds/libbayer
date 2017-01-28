//! Bayer image definitions.

use std::io::Read;
use byteorder::{BigEndian,LittleEndian,ReadBytesExt};

use ::BayerResult;

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
