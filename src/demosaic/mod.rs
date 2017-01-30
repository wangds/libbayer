//! Collection of demosaicing algorithms.

use ::{BayerDepth,RasterDepth};

/// The demosaicing algorithm to use to fill in the missing data.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Demosaic {
    None,
}

pub mod none;

/// Check if the image depth and the raster depth are compatible.
fn check_depth(bayer: BayerDepth, raster: RasterDepth) -> bool {
    match raster {
        RasterDepth::Depth8 =>
            bayer == BayerDepth::Depth8,
        RasterDepth::Depth16 =>
            bayer == BayerDepth::Depth16BE || bayer == BayerDepth::Depth16LE,
    }
}
