//! Collection of demosaicing algorithms.

use ::{BayerDepth,RasterDepth};

/// The demosaicing algorithm to use to fill in the missing data.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Demosaic {
    None,
    NearestNeighbour,
    Linear,
}

macro_rules! rotate {
    ($v0:ident <- $v1:ident) => {{
        let rot = $v0;
        $v0 = $v1;
        $v1 = rot;
    }};
    ($v0:ident <- $v1:ident <- $v2:ident) => {{
        let rot = $v0;
        $v0 = $v1;
        $v1 = $v2;
        $v2 = rot;
    }};
}

pub mod linear;
pub mod nearestneighbour;
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
