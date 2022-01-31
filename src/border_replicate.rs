//! Bayer reader that replicates pixels on the border.
//!
//! If the raw data is given by the unprimed values shown below, this
//! reader will produce the following row, where the primed values
//! have the same value as the unprimed values.
//!
//! ```text
//!   r0' g0' r0' g0' | r0 g0 r1 g1 r2 g2 ... rl gl rm gm rn gn | rn' gn' rn' gn'
//! ```

use std::io::Read;

use crate::bayer::*;
use crate::BayerResult;

/// Tuple structs (x1, x2, x3) designating the different sub-regions
/// of the output lines.
///
/// ```text
///    0 .. x1 => left border
///   x1 .. x2 => raw data
///   x2 .. x3 => right border
/// ```
pub struct BorderReplicate8(usize, usize, usize);
pub struct BorderReplicate16BE(usize, usize, usize);
pub struct BorderReplicate16LE(usize, usize, usize);

macro_rules! fill_row {
    ($dst:ident, $x1:expr, $x2:expr, $x3:expr) => {{
        let mut i;

        // Left border.
        let r0 = $dst[$x1 + 0];
        let g0 = $dst[$x1 + 1];
        i = 0;
        if $x1 % 2 == 1 {
            $dst[0] = g0;
            i = 1;
        }
        while i < $x1 {
            $dst[i + 0] = r0;
            $dst[i + 1] = g0;
            i += 2;
        }

        // Right border.
        let r0 = $dst[$x2 - 2];
        let g0 = $dst[$x2 - 1];
        i = $x2;
        while i + 1 < $x3 {
            $dst[i + 0] = r0;
            $dst[i + 1] = g0;
            i += 2;
        }
        if i == $x3 - 1 {
            $dst[i] = r0;
        }
    }};
}

impl BorderReplicate8 {
    pub fn new(width: usize, padding: usize) -> Self {
        let x1 = padding;
        let x2 = x1.checked_add(width).expect("overflow");
        let x3 = x2.checked_add(padding).expect("overflow");
        assert!(width >= 2);

        BorderReplicate8(x1, x2, x3)
    }
}

impl BayerRead8 for BorderReplicate8 {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u8]) -> BayerResult<()> {
        let BorderReplicate8(x1, x2, x3) = *self;
        read_exact_u8(r, &mut dst[x1..x2])?;
        fill_row!(dst, x1, x2, x3);
        Ok(())
    }
}

impl BorderReplicate16BE {
    pub fn new(width: usize, padding: usize) -> Self {
        let x1 = padding;
        let x2 = x1.checked_add(width).expect("overflow");
        let x3 = x2.checked_add(padding).expect("overflow");
        assert!(width >= 2);

        BorderReplicate16BE(x1, x2, x3)
    }
}

impl BayerRead16 for BorderReplicate16BE {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u16]) -> BayerResult<()> {
        let BorderReplicate16BE(x1, x2, x3) = *self;
        read_exact_u16be(r, &mut dst[x1..x2])?;
        fill_row!(dst, x1, x2, x3);
        Ok(())
    }
}

impl BorderReplicate16LE {
    pub fn new(width: usize, padding: usize) -> Self {
        let x1 = padding;
        let x2 = x1.checked_add(width).expect("overflow");
        let x3 = x2.checked_add(padding).expect("overflow");
        assert!(width >= 2);

        BorderReplicate16LE(x1, x2, x3)
    }
}

impl BayerRead16 for BorderReplicate16LE {
    fn read_line(&self, r: &mut dyn Read, dst: &mut [u16]) -> BayerResult<()> {
        let BorderReplicate16LE(x1, x2, x3) = *self;
        read_exact_u16le(r, &mut dst[x1..x2])?;
        fill_row!(dst, x1, x2, x3);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BorderReplicate8;
    use crate::bayer::BayerRead8;
    use std::io::Cursor;

    #[test]
    fn test_replicate_even() {
        let src = [1, 2, 3, 4, 5, 6];

        let expected = [
            1, 2, 1, 2, /* ----- */ 1, 2, 3, 4, 5, 6, /* -------------------- */ 5, 6, 5, 6,
        ];

        let rdr = BorderReplicate8::new(6, 4);
        let mut buf = [0u8; 4 + 6 + 4];

        let res = rdr.read_line(&mut Cursor::new(&src[..]), &mut buf);
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }

    #[test]
    fn test_replicate_odd() {
        let src = [1, 2, 3, 4, 5];

        let expected = [
            2, 1, 2, /* --- */ 1, 2, 3, 4, 5, /* --------------- */ 4, 5, 4,
        ];

        let rdr = BorderReplicate8::new(5, 3);
        let mut buf = [0u8; 3 + 5 + 3];

        let res = rdr.read_line(&mut Cursor::new(&src[..]), &mut buf);
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }
}
