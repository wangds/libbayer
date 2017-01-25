//! Raster implementation.

use ::RasterMut;

/// Depth of a raster.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum RasterDepth {
    Depth8,
    Depth16,
}

impl<'a> RasterMut<'a> {
    /// Allocate a new raster for the given destination buffer slice.
    ///
    /// # Examples
    ///
    /// ```
    /// const IMG_W: usize = 320;
    /// const IMG_H: usize = 200;
    /// let mut buf = [0; 3 * IMG_W * IMG_H];
    ///
    /// bayer::RasterMut::new(
    ///         IMG_W, IMG_H, bayer::RasterDepth::Depth8,
    ///         &mut buf);
    /// ```
    pub fn new(w: usize, h: usize, depth: RasterDepth, buf: &'a mut [u8])
            -> Self {
        let bytes_per_pixel = depth.bytes_per_pixel();
        let stride = w.checked_mul(bytes_per_pixel).expect("overflow");
        Self::with_offset(0, 0, w, h, stride, depth, buf)
    }

    /// Allocate a new raster for the given destination buffer slice.
    /// Stride is in number of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// const IMG_W: usize = 320;
    /// const IMG_H: usize = 200;
    /// let mut buf = [0; 3 * IMG_W * IMG_H];
    ///
    /// bayer::RasterMut::with_offset(
    ///         0, 0, IMG_W, IMG_H, 3 * IMG_W, bayer::RasterDepth::Depth8,
    ///         &mut buf);
    /// ```
    pub fn with_offset(
            x: usize, y: usize, w: usize, h: usize, stride: usize,
            depth: RasterDepth, buf: &'a mut [u8])
            -> Self {
        let x1 = x.checked_add(w).expect("overflow");
        let y1 = y.checked_add(h).expect("overflow");
        let bytes_per_pixel = depth.bytes_per_pixel();
        assert!(x < x1 && x1.checked_mul(bytes_per_pixel).expect("overflow") <= stride && h > 0);
        assert!(stride.checked_mul(y1).expect("overflow") <= buf.len());
        assert_eq!(stride % bytes_per_pixel, 0);

        RasterMut {
            x: x,
            y: y,
            w: w,
            h: h,
            stride: stride,
            depth: depth,
            buf: buf,
        }
    }
}

impl RasterDepth {
    /// The number of bytes per pixel for a raster of the given depth.
    fn bytes_per_pixel(self) -> usize {
        match self {
            RasterDepth::Depth8 => 3,
            RasterDepth::Depth16 => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use ::RasterMut;
    use super::RasterDepth;

    #[test]
    #[should_panic]
    fn test_raster_mut_overflow() {
        let mut buf = [0; 1];
        let _ = RasterMut::new(
                ::std::usize::MAX, ::std::usize::MAX, RasterDepth::Depth8, &mut buf);
    }
}
