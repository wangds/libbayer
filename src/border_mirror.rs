//! Bayer reader that mirrors pixels on the border.
//!
//! If the raw data is given by the unprimed values shown below, this
//! reader will produce the following row, where the primed values
//! have the same value as the unprimed values.
//!
//! ```text
//!   r2' g1' r1' g0' | r0 g0 r1 g1 r2 g2 ... rl gl rm gm rn gn | rn' gm' rm' gl'
//! ```

use std::io::Read;

use ::BayerResult;
use bayer::*;

/// Tuple structs (x1, x2, x3) designating the different sub-regions
/// of the output lines.
///
/// ```text
///    0 .. x1 => left border
///   x1 .. x2 => raw data
///   x2 .. x3 => right border
/// ```
pub struct BorderMirror8(usize, usize, usize);
pub struct BorderMirror16BE(usize, usize, usize);
pub struct BorderMirror16LE(usize, usize, usize);

macro_rules! fill_row {
    ($dst:ident, $x1:expr, $x2:expr, $x3:expr) => {
        let mut i;
        let mut j;

        // Left border.
        i = $x1;
        j = $x1 + 1;
        while i > 0 {
            $dst[i - 1] = $dst[j];
            i = i - 1;
            j = j + 1;
        }

        // Right border.
        i = $x2;
        j = $x2 - 2;
        while i < $x3 {
            $dst[i] = $dst[j];
            i = i + 1;
            j = j - 1;
        }
    }
}

impl BorderMirror8 {
    #[allow(dead_code)]
    pub fn new(width: usize, padding: usize) -> Self {
        let x1 = padding;
        let x2 = x1.checked_add(width).expect("overflow");
        let x3 = x2.checked_add(padding).expect("overflow");
        assert!(width > padding);

        BorderMirror8(x1, x2, x3)
    }
}

impl BayerRead8 for BorderMirror8 {
    fn read_line(&self, r: &mut Read, dst: &mut [u8])
            -> BayerResult<()> {
        let BorderMirror8(x1, x2, x3) = *self;
        read_exact_u8(r, &mut dst[x1..x2])?;
        fill_row!(dst, x1, x2, x3);
        Ok(())
    }
}

impl BorderMirror16BE {
    #[allow(dead_code)]
    pub fn new(width: usize, padding: usize) -> Self {
        let x1 = padding;
        let x2 = x1.checked_add(width).expect("overflow");
        let x3 = x2.checked_add(padding).expect("overflow");
        assert!(width > padding);

        BorderMirror16BE(x1, x2, x3)
    }
}

impl BayerRead16 for BorderMirror16BE {
    fn read_line(&self, r: &mut Read, dst: &mut [u16])
            -> BayerResult<()> {
        let BorderMirror16BE(x1, x2, x3) = *self;
        read_exact_u16be(r, &mut dst[x1..x2])?;
        fill_row!(dst, x1, x2, x3);
        Ok(())
    }
}

impl BorderMirror16LE {
    #[allow(dead_code)]
    pub fn new(width: usize, padding: usize) -> Self {
        let x1 = padding;
        let x2 = x1.checked_add(width).expect("overflow");
        let x3 = x2.checked_add(padding).expect("overflow");
        assert!(width > padding);

        BorderMirror16LE(x1, x2, x3)
    }
}

impl BayerRead16 for BorderMirror16LE {
    fn read_line(&self, r: &mut Read, dst: &mut [u16])
            -> BayerResult<()> {
        let BorderMirror16LE(x1, x2, x3) = *self;
        read_exact_u16le(r, &mut dst[x1..x2])?;
        fill_row!(dst, x1, x2, x3);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use bayer::BayerRead8;
    use super::BorderMirror8;

    #[test]
    fn test_mirror_even() {
        let src = [
            1,2, 3,4, 5,6 ];

        let expected = [
            5,4, 3,2,
            /*-----*/ 1,2, 3,4, 5,6,
            /*--------------------*/ 5,4, 3,2 ];

        let rdr = BorderMirror8::new(6, 4);
        let mut buf = [0u8; 4 + 6 + 4];

        let res = rdr.read_line(&mut Cursor::new(&src[..]), &mut buf);
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }

    #[test]
    fn test_mirror_odd() {
        let src = [
            1,2, 3,4, 5, ];

        let expected = [
            4, 3,2,
            /*---*/ 1,2, 3,4, 5,
            /*---------------*/ 4, 3,2 ];

        let rdr = BorderMirror8::new(5, 3);
        let mut buf = [0u8; 3 + 5 + 3];

        let res = rdr.read_line(&mut Cursor::new(&src[..]), &mut buf);
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }
}
