//! Bayer reader without any additional border logic.

use std::io::Read;

use ::BayerResult;
use bayer::*;

pub struct BorderNone8;
pub struct BorderNone16BE;
pub struct BorderNone16LE;

impl BorderNone8 {
    #[allow(dead_code)]
    pub fn new() -> Self {
        BorderNone8
    }
}

impl BayerRead8 for BorderNone8 {
    fn read_line(&self, r: &mut Read, dst: &mut [u8])
            -> BayerResult<()> {
        read_exact_u8(r, dst)
    }
}

impl BorderNone16BE {
    #[allow(dead_code)]
    pub fn new() -> Self {
        BorderNone16BE
    }
}

impl BayerRead16 for BorderNone16BE {
    fn read_line(&self, r: &mut Read, dst: &mut [u16])
            -> BayerResult<()> {
        read_exact_u16be(r, dst)
    }
}

impl BorderNone16LE {
    #[allow(dead_code)]
    pub fn new() -> Self {
        BorderNone16LE
    }
}

impl BayerRead16 for BorderNone16LE {
    fn read_line(&self, r: &mut Read, dst: &mut [u16])
            -> BayerResult<()> {
        read_exact_u16le(r, dst)
    }
}
