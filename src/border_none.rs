//! Bayer reader without any additional border logic.

use std::io::Read;

use crate::bayer::*;
use crate::BayerResult;

pub struct BorderNone8;
pub struct BorderNone16BE;
pub struct BorderNone16LE;

impl BorderNone8 {
    pub fn new() -> Self {
        BorderNone8
    }
}

impl BayerRead8 for BorderNone8 {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u8]) -> BayerResult<()> {
        read_exact_u8(r, dst)
    }
}

impl BorderNone16BE {
    pub fn new() -> Self {
        BorderNone16BE
    }
}

impl BayerRead16 for BorderNone16BE {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u16]) -> BayerResult<()> {
        read_exact_u16be(r, dst)
    }
}

impl BorderNone16LE {
    pub fn new() -> Self {
        BorderNone16LE
    }
}

impl BayerRead16 for BorderNone16LE {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u16]) -> BayerResult<()> {
        read_exact_u16le(r, dst)
    }
}
