//! Raster implementation.

use std::slice;

use crate::RasterMut;

/// Depth of a raster.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// bayer::RasterMut::new(IMG_W, IMG_H, bayer::RasterDepth::Depth8, &mut buf);
    /// ```
    pub fn new(w: usize, h: usize, depth: RasterDepth, buf: &'a mut [u8]) -> Self {
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
    ///     0,
    ///     0,
    ///     IMG_W,
    ///     IMG_H,
    ///     3 * IMG_W,
    ///     bayer::RasterDepth::Depth8,
    ///     &mut buf,
    /// );
    /// ```
    pub fn with_offset(
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        stride: usize,
        depth: RasterDepth,
        buf: &'a mut [u8],
    ) -> Self {
        let x1 = x.checked_add(w).expect("overflow");
        let y1 = y.checked_add(h).expect("overflow");
        let bytes_per_pixel = depth.bytes_per_pixel();
        assert!(x < x1 && x1.checked_mul(bytes_per_pixel).expect("overflow") <= stride && h > 0);
        assert!(stride.checked_mul(y1).expect("overflow") <= buf.len());
        assert_eq!(stride % bytes_per_pixel, 0);

        RasterMut {
            x,
            y,
            w,
            h,
            stride,
            depth,
            buf,
        }
    }

    /// Borrow a mutable [`u8`] row slice.
    ///
    /// # Panics
    ///
    /// Panics if the raster is not 8 bit per pixel.
    pub fn borrow_row_u8_mut(&mut self, y: usize) -> &mut [u8] {
        assert!(self.depth == RasterDepth::Depth8);
        assert!(y < self.h);

        let bytes_per_pixel = 3;
        let start = self.stride * (self.y + y) + bytes_per_pixel * self.x;
        let end = start + bytes_per_pixel * self.w;
        &mut self.buf[start..end]
    }

    /// Borrow a mutable [`u16`] row slice.
    ///
    /// # Panics
    ///
    /// Panics if the raster is not 16 bit per pixel.
    pub fn borrow_row_u16_mut(&mut self, y: usize) -> &mut [u16] {
        assert!(self.depth == RasterDepth::Depth16);
        assert!(y < self.h);

        let bytes_per_pixel = 6;
        let start = self.stride * (self.y + y) + bytes_per_pixel * self.x;
        let end = start + bytes_per_pixel * self.w;
        let s = &mut self.buf[start..end];

        unsafe { slice::from_raw_parts_mut(s.as_mut_ptr() as *mut u16, 3 * self.w) }
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
    use super::RasterDepth;
    use crate::RasterMut;

    #[test]
    #[should_panic]
    fn test_raster_mut_overflow() {
        let mut buf = [0; 1];
        let _ = RasterMut::new(
            ::std::usize::MAX,
            ::std::usize::MAX,
            RasterDepth::Depth8,
            &mut buf,
        );
    }

    #[test]
    fn test_borrow_row_u16_mut() {
        let expected = [
            0x00, 0x00, 0x01, 0x01, 0x02, 0x02, 0x03, 0x03, 0x04, 0x04, 0x05, 0x05, 0x06, 0x06,
            0x07, 0x07, 0x08, 0x08, 0x09, 0x09, 0x0A, 0x0A, 0x0B, 0x0B,
        ];

        const IMG_W: usize = 4;
        const IMG_H: usize = 1;
        let mut buf = [0u8; 6 * IMG_W * IMG_H];

        {
            let mut dst = RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth16, &mut buf);
            let row = dst.borrow_row_u16_mut(0);

            for (i, elt) in row.iter_mut().enumerate() {
                // Work around different endians.
                let i = i as u16;
                *elt = (i << 8) | i;
            }
        }

        assert_eq!(&buf[0..6 * IMG_W * IMG_H], &expected[..]);
    }
}
