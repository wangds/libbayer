//! Demosaicing using nearest neighbour interpolation.

use std::io::Read;

use ::{BayerDepth,BayerError,BayerResult,CFA,RasterMut};
use bayer::{BayerRead8,BayerRead16};
use border_replicate::*;
use demosaic::check_depth;

const PADDING: usize = 1;

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
    ($row:ident, $prev:expr, $curr:expr, $cfa:expr, $w:expr) => {{
        let mut cfa_x = $cfa;
        for i in 0..$w {
            match cfa_x {
                CFA::BGGR | CFA::RGGB =>
                    apply_kernel_c!($row, $prev, $curr, cfa_x, i),
                CFA::GBRG | CFA::GRBG =>
                    apply_kernel_g!($row, $prev, $curr, cfa_x, i),
            }
            cfa_x = cfa_x.next_x();
        }
    }}
}

macro_rules! apply_kernel_c {
    ($row:ident, $prev:expr, $curr:expr, $cfa:expr, $i:expr) => {{
        // current = B/R, diagonal = R/B.
        let (c, d) = if $cfa == CFA::BGGR { (2, 0) } else { (0, 2) };
        let j = $i + PADDING;

        $row[3 * $i + c] = $curr[j];
        $row[3 * $i + 1] = $curr[j - 1];
        $row[3 * $i + d] = $prev[j - 1];
    }}
}

macro_rules! apply_kernel_g {
    ($row:ident, $prev:expr, $curr:expr, $cfa:expr, $i:expr) => {{
        // horizontal = B/R, vertical = R/G.
        let (h, v) = if $cfa == CFA::GBRG { (2, 0) } else { (0, 2) };
        let j = $i + PADDING;

        $row[3 * $i + h] = $curr[j - 1];
        $row[3 * $i + 1] = $curr[j];
        $row[3 * $i + v] = $prev[j];
    }}
}

/*--------------------------------------------------------------*/

fn debayer_u8(r: &mut Read, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut prev = vec![0u8; 2 * PADDING + w];
    let mut curr = vec![0u8; 2 * PADDING + w];
    let mut cfa = cfa;

    let rdr = BorderReplicate8::new(w, PADDING);
    rdr.read_line(r, &mut prev)?;
    rdr.read_line(r, &mut curr)?;

    {   // y = 0.
        let row = dst.borrow_row_u8_mut(0);
        apply_kernel_row!(row, curr, prev, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = 1.
        let row = dst.borrow_row_u8_mut(1);
        apply_kernel_row!(row, prev, curr, cfa, w);
        cfa = cfa.next_y();
    }

    for y in 2..h {
        rotate!(prev <- curr);
        rdr.read_line(r, &mut curr)?;

        let row = dst.borrow_row_u8_mut(y);
        apply_kernel_row!(row, prev, curr, cfa, w);
        cfa = cfa.next_y();
    }

    Ok(())
}

fn debayer_u16(r: &mut Read, be: bool, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut prev = vec![0u16; 2 * PADDING + w];
    let mut curr = vec![0u16; 2 * PADDING + w];
    let mut cfa = cfa;

    let rdr: Box<BayerRead16> = if be {
        Box::new(BorderReplicate16BE::new(w, PADDING))
    } else {
        Box::new(BorderReplicate16LE::new(w, PADDING))
    };
    rdr.read_line(r, &mut prev)?;
    rdr.read_line(r, &mut curr)?;

    {   // y = 0.
        let row = dst.borrow_row_u16_mut(0);
        apply_kernel_row!(row, curr, prev, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = 1.
        let row = dst.borrow_row_u16_mut(1);
        apply_kernel_row!(row, prev, curr, cfa, w);
        cfa = cfa.next_y();
    }

    for y in 2..h {
        rotate!(prev <- curr);
        rdr.read_line(r, &mut curr)?;

        let row = dst.borrow_row_u16_mut(y);
        apply_kernel_row!(row, prev, curr, cfa, w);
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
            229, 67, 51,  229, 67, 51,   95, 67, 51,   95,146,241,
            229,232, 51,  229,232, 51,   95,229, 51,   95,229,241,
            169,161, 51,  169,161, 51,   15,161, 51,   15, 52,241,
            169, 45,175,  169, 45,175,   15, 98,175,   15, 98,197 ];

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
            229, 67,232,  229, 67,232,   95, 67,232,
            229,146,232,  229,146,232,   95, 51,232,
            229,241,232,  229,241,232,  169,241,232 ];

        const IMG_W: usize = 3;
        const IMG_H: usize = 3;
        let mut buf = [0u8; 3 * IMG_W * IMG_H];

        let res = debayer_u8(&mut Cursor::new(&src[..]), CFA::RGGB,
                &mut RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut buf));
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }
}
