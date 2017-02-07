//! Demosaicing without any interpolation.

use std::io::Read;

use ::{BayerDepth,BayerError,BayerResult,CFA,RasterMut};
use bayer::{BayerRead8,BayerRead16};
use border_none::*;
use demosaic::check_depth;

pub fn run(r: &mut Read,
        depth: BayerDepth, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    if dst.w < 2 || dst.h < 2 {
        return Err(BayerError::WrongResolution);
    }
    if !check_depth(depth, dst.depth) {
        return Err(BayerError::WrongDepth);
    }

    match depth {
        BayerDepth::Depth8 => debayer_u8(r, cfa, dst),
        BayerDepth::Depth16BE => debayer_u16(r, true, cfa, dst),
        BayerDepth::Depth16LE => debayer_u16(r, false, cfa, dst),
    }
}

macro_rules! apply_kernel_row {
    ($row:ident, $curr:expr, $cfa:expr, $w:expr) => {{
        for e in $row.iter_mut() {
            *e = 0;
        }

        let (mut i, cfa_c) =
            if $cfa == CFA::BGGR || $cfa == CFA::RGGB {
                (0, $cfa)
            } else {
                apply_kernel_g!($row, $curr, 0);
                (1, $cfa.next_x())
            };

        while i + 1 < $w {
            apply_kernel_c!($row, $curr, cfa_c, i);
            apply_kernel_g!($row, $curr, i + 1);
            i = i + 2;
        }

        if i < $w {
            apply_kernel_c!($row, $curr, cfa_c, i);
        }
    }}
}

macro_rules! apply_kernel_c {
    ($row:ident, $curr:expr, $cfa:expr, $i:expr) => {{
        if $cfa == CFA::BGGR {
            $row[3 * $i + 2] = $curr[$i];
        } else {
            $row[3 * $i + 0] = $curr[$i];
        }
    }}
}

macro_rules! apply_kernel_g {
    ($row:ident, $curr:expr, $i:expr) => {{
        $row[3 * $i + 1] = $curr[$i];
    }}
}

/*--------------------------------------------------------------*/

fn debayer_u8(r: &mut Read, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut curr = vec![0u8; w];
    let mut cfa = cfa;

    let rdr = BorderNone8::new();

    for y in 0..h {
        let row = dst.borrow_row_u8_mut(y);
        rdr.read_line(r, &mut curr)?;
        apply_kernel_row!(row, curr, cfa, w);
        cfa = cfa.next_y();
    }

    Ok(())
}

fn debayer_u16(r: &mut Read, be: bool, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut curr = vec![0u16; w];
    let mut cfa = cfa;

    let rdr: Box<BayerRead16> = if be {
        Box::new(BorderNone16BE::new())
    } else {
        Box::new(BorderNone16LE::new())
    };

    for y in 0..h {
        let row = dst.borrow_row_u16_mut(y);
        rdr.read_line(r, &mut curr)?;
        apply_kernel_row!(row, curr, cfa, w);
        cfa = cfa.next_y();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use ::{CFA,RasterDepth,RasterMut};
    use super::debayer_u8;

    #[test]
    fn test_even() {
        // R: set.seed(0); matrix(floor(runif(n=16, min=0, max=256)), nrow=4, byrow=TRUE)
        let src = [
            229, 67, 95,146,
            232, 51,229,241,
            169,161, 15, 52,
             45,175, 98,197 ];

        let expected = [
            229,  0,  0,    0, 67,  0,   95,  0,  0,    0,146,  0,
              0,232,  0,    0,  0, 51,    0,229,  0,    0,  0,241,
            169,  0,  0,    0,161,  0,   15,  0,  0,    0, 52 , 0,
              0, 45,  0,    0,  0,175,    0, 98,  0,    0,  0,197 ];

        const IMG_W: usize = 4;
        const IMG_H: usize = 4;
        let mut buf = [0u8; 3 * IMG_W * IMG_H];

        let res = debayer_u8(&mut Cursor::new(&src[..]), CFA::RGGB,
                &mut RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut buf));
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }

    #[test]
    fn test_odd() {
        // R: set.seed(0); matrix(floor(runif(n=9, min=0, max=256)), nrow=3, byrow=TRUE)
        let src = [
            229, 67, 95,
            146,232, 51,
            229,241,169 ];

        let expected = [
            229,  0,  0,    0, 67,  0,   95,  0,  0,
              0,146,  0,    0,  0,232,    0, 51,  0,
            229,  0,  0,    0,241,  0,  169,  0,  0 ];

        const IMG_W: usize = 3;
        const IMG_H: usize = 3;
        let mut buf = [0u8; 3 * IMG_W * IMG_H];

        let res = debayer_u8(&mut Cursor::new(&src[..]), CFA::RGGB,
                &mut RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut buf));
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }
}
