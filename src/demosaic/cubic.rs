//! Demosaicing using cubic interpolation.
//!
//! ```text
//!   green_kernel = (1 / 256) *
//!       [   0   0   0   1   0   0   0
//!       ;   0   0  -9   0  -9   0   0
//!       ;   0  -9   0  81   0  -9   0
//!       ;   1   0  81 256  81   0   1
//!       ;   0  -9   0  81   0  -9   0
//!       ;   0   0  -9   0  -9   0   0
//!       ;   0   0   0   1   0   0   0 ];
//!
//!   red/blue_kernel = (1 / 256) *
//!       [   1   0  -9 -16  -9   0   1
//!       ;   0   0   0   0   0   0   0
//!       ;  -9   0  81 144  81   0  -9
//!       ; -16   0 144 256 144   0 -16
//!       ;  -9   0  81 144  81   0  -9
//!       ;   0   0   0   0   0   0   0
//!       ;   1   0  -9 -16  -9   0   1 ];
//! ```

use std::cmp::min;
use std::io::Read;

#[cfg(feature = "rayon")]
use std::slice;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use ::{BayerDepth,BayerError,BayerResult,CFA,RasterMut};
use bayer::{BayerRead8,BayerRead16};
use border_mirror::*;
use demosaic::check_depth;

const PADDING: usize = 3;

pub fn run(r: &mut Read,
        depth: BayerDepth, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    if dst.w < 4 || dst.h < 4 {
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
    ($T:ident; $row:ident,
            $prv3:expr, $prv2:expr, $prv1:expr, $curr:expr,
            $nxt1:expr, $nxt2:expr, $nxt3:expr,
            $cfa:expr, $w:expr) => {{
        let (mut i, cfa_c, cfa_g) =
            if $cfa == CFA::BGGR || $cfa == CFA::RGGB {
                (0, $cfa, $cfa.next_x())
            } else {
                apply_kernel_g!($T; $row, $w, $prv3, $prv2, $prv1, $curr, $nxt1, $nxt2, $nxt3, $cfa, 0);
                (1, $cfa.next_x(), $cfa)
            };

        while i + 1 < $w {
            apply_kernel_c!($T; $row, $w, $prv3, $prv2, $prv1, $curr, $nxt1, $nxt2, $nxt3, cfa_c, i);
            apply_kernel_g!($T; $row, $w, $prv3, $prv2, $prv1, $curr, $nxt1, $nxt2, $nxt3, cfa_g, i + 1);
            i = i + 2;
        }

        if i < $w {
            apply_kernel_c!($T; $row, $w, $prv3, $prv2, $prv1, $curr, $nxt1, $nxt2, $nxt3, cfa_c, i);
        }
    }}
}

macro_rules! apply_kernel_c {
    ($T:ident; $row:ident, $w:expr,
            $prv3:expr, $prv2:expr, $prv1:expr, $curr:expr,
            $nxt1:expr, $nxt2:expr, $nxt3:expr,
            $cfa:expr, $i:expr) => {{
        // current = B/R, diagonal = R/B.
        let (c, d) = if $cfa == CFA::BGGR { (2, 0) } else { (0, 2) };
        let j = $i + PADDING;

        let g_pos
            = (   $prv1[j] as u32
                  + $curr[j - 1] as u32 + $curr[j + 1] as u32
                  + $nxt1[j] as u32) * 81
            + (   $prv3[j] as u32
                  + $curr[j - 3] as u32 + $curr[j + 3] as u32
                  + $nxt3[j] as u32);
        let g_neg
            = (   $prv2[j - 1] as u32 + $prv2[j + 1] as u32
                  + $prv1[j - 2] as u32 + $prv1[j + 2] as u32
                  + $nxt1[j - 2] as u32 + $nxt1[j + 2] as u32
                  + $nxt2[j - 1] as u32 + $nxt2[j + 1] as u32) * 9;

        let d_pos
            = (   $prv1[j - 1] as u32 + $prv1[j + 1] as u32
                  + $nxt1[j - 1] as u32 + $nxt1[j + 1] as u32) * 81
            + (   $prv3[j - 3] as u32 + $prv3[j + 3] as u32
                  + $nxt3[j - 3] as u32 + $nxt3[j + 3] as u32);
        let d_neg
            = (   $prv3[j - 1] as u32 + $prv3[j + 1] as u32
                  + $prv1[j - 3] as u32 + $prv1[j + 3] as u32
                  + $nxt1[j - 3] as u32 + $nxt1[j + 3] as u32
                  + $nxt3[j - 1] as u32 + $nxt3[j + 1] as u32) * 9;

        $row[3 * $i + c] = $curr[j];
        $row[3 * $i + 1]
            = min(g_pos.saturating_sub(g_neg) / 256,
                    $T::max_value() as u32) as $T;
        $row[3 * $i + d]
            = min(d_pos.saturating_sub(d_neg) / 256,
                    $T::max_value() as u32) as $T;
    }}
}

macro_rules! apply_kernel_g {
    ($T:ident; $row:ident, $w:expr,
            $prv3:expr, $prv2:expr, $prv1:expr, $curr:expr,
            $nxt1:expr, $nxt2:expr, $nxt3:expr,
            $cfa:expr, $i:expr) => {{
        // horizontal = B/R, vertical = R/G.
        let (h, v) = if $cfa == CFA::GBRG { (2, 0) } else { (0, 2) };
        let j = $i + PADDING;

        let h_pos = ($curr[j - 1] as u32 + $curr[j + 1] as u32) * 9;
        let h_neg = ($curr[j - 3] as u32 + $curr[j + 3] as u32);
        let v_pos = ($prv1[j] as u32 + $nxt1[j] as u32) * 9;
        let v_neg = ($prv3[j] as u32 + $nxt3[j] as u32);

        $row[3 * $i + h]
            = min(h_pos.saturating_sub(h_neg) / 16,
                    $T::max_value() as u32) as $T;
        $row[3 * $i + 1] = $curr[j];
        $row[3 * $i + v]
            = min(v_pos.saturating_sub(v_neg) / 16,
                    $T::max_value() as u32) as $T;
    }}
}

/*--------------------------------------------------------------*/
/* Rayon                                                        */
/*--------------------------------------------------------------*/

#[cfg(feature = "rayon")]
#[allow(unused_parens)]
fn debayer_u8(r: &mut Read, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut data = vec![0u8; (2 * PADDING + w) * (2 * PADDING + h)];

    // Read all data.
    {
        let stride = 2 * PADDING + w;
        let rdr = BorderMirror8::new(w, PADDING);
        for mut row in data.chunks_mut(stride).skip(PADDING).take(h) {
            rdr.read_line(r, &mut row)?;
        }

        {
            let (top, src) = data.split_at_mut(stride * PADDING);
            top[(stride * 0)..(stride * 1)].copy_from_slice(&src[(stride * 3)..(stride * 4)]);
            top[(stride * 1)..(stride * 2)].copy_from_slice(&src[(stride * 2)..(stride * 3)]);
            top[(stride * 2)..(stride * 3)].copy_from_slice(&src[(stride * 1)..(stride * 2)]);
        }

        {
            let (src, bottom) = data.split_at_mut(stride * (h + PADDING));
            let yy = PADDING + h;
            bottom[(stride * 0)..(stride * 1)].copy_from_slice(&src[(stride * (yy - 2))..(stride * (yy - 1))]);
            bottom[(stride * 1)..(stride * 2)].copy_from_slice(&src[(stride * (yy - 3))..(stride * (yy - 2))]);
            bottom[(stride * 2)..(stride * 3)].copy_from_slice(&src[(stride * (yy - 4))..(stride * (yy - 3))]);
        }
    }

    dst.buf.par_chunks_mut(dst.stride).enumerate()
            .for_each(|(y, mut row)| {
        let stride = 2 * PADDING + w;
        let prv3 = &data[(stride * (PADDING + y - 3)) .. (stride * (PADDING + y - 2))];
        let prv2 = &data[(stride * (PADDING + y - 2)) .. (stride * (PADDING + y - 1))];
        let prv1 = &data[(stride * (PADDING + y - 1)) .. (stride * (PADDING + y + 0))];
        let curr = &data[(stride * (PADDING + y + 0)) .. (stride * (PADDING + y + 1))];
        let nxt1 = &data[(stride * (PADDING + y + 1)) .. (stride * (PADDING + y + 2))];
        let nxt2 = &data[(stride * (PADDING + y + 2)) .. (stride * (PADDING + y + 3))];
        let nxt3 = &data[(stride * (PADDING + y + 3)) .. (stride * (PADDING + y + 4))];
        let cfa_y = if y % 2 == 0 { cfa } else { cfa.next_y() };

        apply_kernel_row!(u8; row, prv3, prv2, prv1, curr, nxt1, nxt2, nxt3, cfa_y, w);
    });

    Ok(())
}

#[cfg(feature = "rayon")]
#[allow(unused_parens)]
fn debayer_u16(r: &mut Read, be: bool, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut data = vec![0u16; (2 * PADDING + w) * (2 * PADDING + h)];

    // Read all data.
    {
        let stride = 2 * PADDING + w;
        let rdr: Box<BayerRead16> = if be {
            Box::new(BorderMirror16BE::new(w, PADDING))
        } else {
            Box::new(BorderMirror16LE::new(w, PADDING))
        };

        for mut row in data.chunks_mut(stride).skip(PADDING).take(h) {
            rdr.read_line(r, &mut row)?;
        }

        {
            let (top, src) = data.split_at_mut(stride * PADDING);
            top[(stride * 0)..(stride * 1)].copy_from_slice(&src[(stride * 3)..(stride * 4)]);
            top[(stride * 1)..(stride * 2)].copy_from_slice(&src[(stride * 2)..(stride * 3)]);
            top[(stride * 2)..(stride * 3)].copy_from_slice(&src[(stride * 1)..(stride * 2)]);
        }

        {
            let (src, bottom) = data.split_at_mut(stride * (h + PADDING));
            let yy = PADDING + h;
            bottom[(stride * 0)..(stride * 1)].copy_from_slice(&src[(stride * (yy - 2))..(stride * (yy - 1))]);
            bottom[(stride * 1)..(stride * 2)].copy_from_slice(&src[(stride * (yy - 3))..(stride * (yy - 2))]);
            bottom[(stride * 2)..(stride * 3)].copy_from_slice(&src[(stride * (yy - 4))..(stride * (yy - 3))]);
        }
    }

    dst.buf.par_chunks_mut(dst.stride).enumerate()
            .for_each(|(y, mut row)| {
        let stride = 2 * PADDING + w;
        let prv3 = &data[(stride * (PADDING + y - 3)) .. (stride * (PADDING + y - 2))];
        let prv2 = &data[(stride * (PADDING + y - 2)) .. (stride * (PADDING + y - 1))];
        let prv1 = &data[(stride * (PADDING + y - 1)) .. (stride * (PADDING + y + 0))];
        let curr = &data[(stride * (PADDING + y + 0)) .. (stride * (PADDING + y + 1))];
        let nxt1 = &data[(stride * (PADDING + y + 1)) .. (stride * (PADDING + y + 2))];
        let nxt2 = &data[(stride * (PADDING + y + 2)) .. (stride * (PADDING + y + 3))];
        let nxt3 = &data[(stride * (PADDING + y + 3)) .. (stride * (PADDING + y + 4))];
        let cfa_y = if y % 2 == 0 { cfa } else { cfa.next_y() };

        let row16 = unsafe{ slice::from_raw_parts_mut(row.as_mut_ptr() as *mut u16, row.len() / 2) };
        apply_kernel_row!(u16; row16, prv3, prv2, prv1, curr, nxt1, nxt2, nxt3, cfa_y, w);
    });

    Ok(())
}

/*--------------------------------------------------------------*/
/* Naive                                                        */
/*--------------------------------------------------------------*/

#[cfg(not(feature = "rayon"))]
#[allow(unused_parens)]
fn debayer_u8(r: &mut Read, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut prv3 = vec![0u8; 2 * PADDING + w];
    let mut prv2 = vec![0u8; 2 * PADDING + w];
    let mut prv1 = vec![0u8; 2 * PADDING + w];
    let mut curr = vec![0u8; 2 * PADDING + w];
    let mut nxt1 = vec![0u8; 2 * PADDING + w];
    let mut nxt2 = vec![0u8; 2 * PADDING + w];
    let mut nxt3 = vec![0u8; 2 * PADDING + w];
    let mut cfa = cfa;

    let rdr = BorderMirror8::new(w, PADDING);
    rdr.read_line(r, &mut curr)?;
    rdr.read_line(r, &mut nxt1)?;
    rdr.read_line(r, &mut nxt2)?;
    rdr.read_line(r, &mut nxt3)?;

    prv1.copy_from_slice(&nxt1);
    prv2.copy_from_slice(&nxt2);
    prv3.copy_from_slice(&nxt3);

    {   // y = 0.
        let row = dst.borrow_row_u8_mut(0);
        apply_kernel_row!(u8; row, nxt3, nxt2, nxt1, curr, nxt1, nxt2, nxt3, cfa, w);
        cfa = cfa.next_y();
    }

    for y in 1..(h - 3) {
        rotate!(prv3 <- prv2 <- prv1 <- curr <- nxt1 <- nxt2 <- nxt3);
        rdr.read_line(r, &mut nxt3)?;

        let row = dst.borrow_row_u8_mut(y);
        apply_kernel_row!(u8; row, prv3, prv2, prv1, curr, nxt1, nxt2, nxt3, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = h - 3.
        let row = dst.borrow_row_u8_mut(h - 3);
        apply_kernel_row!(u8; row, prv2, prv1, curr, nxt1, nxt2, nxt3, nxt2, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = h - 2.
        let row = dst.borrow_row_u8_mut(h - 2);
        apply_kernel_row!(u8; row, prv1, curr, nxt1, nxt2, nxt3, nxt2, nxt1, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = h - 1.
        let row = dst.borrow_row_u8_mut(h - 1);
        apply_kernel_row!(u8; row, curr, nxt1, nxt2, nxt3, nxt2, nxt1, curr, cfa, w);
    }

    Ok(())
}

#[cfg(not(feature = "rayon"))]
#[allow(unused_parens)]
fn debayer_u16(r: &mut Read, be: bool, cfa: CFA, dst: &mut RasterMut)
        -> BayerResult<()> {
    let (w, h) = (dst.w, dst.h);
    let mut prv3 = vec![0u16; 2 * PADDING + w];
    let mut prv2 = vec![0u16; 2 * PADDING + w];
    let mut prv1 = vec![0u16; 2 * PADDING + w];
    let mut curr = vec![0u16; 2 * PADDING + w];
    let mut nxt1 = vec![0u16; 2 * PADDING + w];
    let mut nxt2 = vec![0u16; 2 * PADDING + w];
    let mut nxt3 = vec![0u16; 2 * PADDING + w];
    let mut cfa = cfa;

    let rdr: Box<BayerRead16> = if be {
        Box::new(BorderMirror16BE::new(w, PADDING))
    } else {
        Box::new(BorderMirror16LE::new(w, PADDING))
    };
    rdr.read_line(r, &mut curr)?;
    rdr.read_line(r, &mut nxt1)?;
    rdr.read_line(r, &mut nxt2)?;
    rdr.read_line(r, &mut nxt3)?;

    prv1.copy_from_slice(&nxt1);
    prv2.copy_from_slice(&nxt2);
    prv3.copy_from_slice(&nxt3);

    {   // y = 0.
        let row = dst.borrow_row_u16_mut(0);
        apply_kernel_row!(u16; row, nxt3, nxt2, nxt1, curr, nxt1, nxt2, nxt3, cfa, w);
        cfa = cfa.next_y();
    }

    for y in 1..(h - 3) {
        rotate!(prv3 <- prv2 <- prv1 <- curr <- nxt1 <- nxt2 <- nxt3);
        rdr.read_line(r, &mut nxt3)?;

        let row = dst.borrow_row_u16_mut(y);
        apply_kernel_row!(u16; row, prv3, prv2, prv1, curr, nxt1, nxt2, nxt3, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = h - 3.
        let row = dst.borrow_row_u16_mut(h - 3);
        apply_kernel_row!(u16; row, prv2, prv1, curr, nxt1, nxt2, nxt3, nxt2, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = h - 2.
        let row = dst.borrow_row_u16_mut(h - 2);
        apply_kernel_row!(u16; row, prv1, curr, nxt1, nxt2, nxt3, nxt2, nxt1, cfa, w);
        cfa = cfa.next_y();
    }

    {   // y = h - 1.
        let row = dst.borrow_row_u16_mut(h - 1);
        apply_kernel_row!(u16; row, curr, nxt1, nxt2, nxt3, nxt2, nxt1, curr, cfa, w);
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
        // R: set.seed(0); matrix(floor(runif(n=64, min=0, max=256)), nrow=8, byrow=TRUE)
        let src = [
            229, 67, 95,146,232, 51,229,241,
            169,161, 15, 52, 45,175, 98,197,
            127,183,253, 97,199,239, 54,166,
             32, 68, 98,  3, 97,222, 87,123,
            153,126, 47,211,171,203, 27,185,
            105,210,165,200,141,135,202,  5,
            122,187,177,122,220,112, 62, 18,
             25, 80,132,169,104,233, 75,117 ];

        let expected = [
			229,122,186,  161, 67,172,   95, 43,108,  155,146, 58,  232, 61,104,  239, 51,169,  229,117,196,  228,241,206,
			182,169,174,  177,110,161,  177, 15, 98,  201, 70, 52,  219, 45,105,  189,104,175,  154, 98,195,  145,159,197,
			127,159,116,  185,183,105,  253, 95, 48,  242, 97, 15,  199,121,106,  123,239,203,   54,153,195,   35,166,167,
			135, 32, 76,  140,103, 68,  151, 98, 21,  176,121,  3,  179, 97,114,  105,159,222,   27, 87,180,    8,115,123,
			153, 80,146,   98,126,141,   47,157,116,  111,211,100,  171,168,142,  106,203,175,   27,179,110,    9,185, 52,
			139,105,211,  115,154,210,   99,165,209,  153,167,200,  193,141,175,  124,179,135,   42,202, 57,   23,160,  5,
			122,118,139,  143,187,145,  177,158,170,  211,122,194,  220,110,200,  143,112,184,   62, 94,114,   42, 18, 60,
			118, 25, 68,  148,129, 80,  193,132,120,  224,111,169,  226,104,213,  148, 95,233,   66, 75,171,   46, 16,117 ];

        const IMG_W: usize = 8;
        const IMG_H: usize = 8;
        let mut buf = [0u8; 3 * IMG_W * IMG_H];

        let res = debayer_u8(&mut Cursor::new(&src[..]), CFA::RGGB,
                &mut RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut buf));
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }

    #[test]
    fn test_odd() {
        // R: set.seed(0); matrix(floor(runif(n=49, min=0, max=256)), nrow=7, byrow=TRUE)
        let src = [
            229, 67, 95,146,232, 51,229,
            241,169,161, 15, 52, 45,175,
             98,197,127,183,253, 97,199,
            239, 54,166, 32, 68, 98,  3,
             97,222, 87,123,153,126, 47,
            211,171,203, 27,185,105,210,
            165,200,141,135,202,  5,122 ];

        let expected = [
			229,147,204,  161, 67,183,   95,123, 96,  155,146, 12,  232, 52, 14,  238, 51, 38,  229,123, 41,
			171,241,188,  136,163,169,  111,161, 90,  177,144, 15,  247, 52, 20,  243, 93, 45,  225,175, 48,
			 98,236,114,  102,197,104,  127,185, 61,  195,183, 23,  253, 95, 42,  230, 97, 71,  199, 99, 76,
			 85,239, 56,   88,208, 54,  105,166, 38,  160,129, 32,  201, 68, 63,  159, 53, 98,  116,  3,106,
			 97,231,114,   88,222,105,   87,178, 63,  126,123, 30,  153,125, 63,   97,126,104,   47,124,114,
			135,211,189,  122,214,171,  114,203, 94,  149,165, 27,  174,185, 57,  124,138,105,   79,210,114,
			165,203,205,  150,200,185,  141,184,101,  175,135, 26,  202,116, 56,  160,  5,105,  122, 93,115 ];

        const IMG_W: usize = 7;
        const IMG_H: usize = 7;
        let mut buf = [0u8; 3 * IMG_W * IMG_H];

        let res = debayer_u8(&mut Cursor::new(&src[..]), CFA::RGGB,
                &mut RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut buf));
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }

    #[test]
    fn test_overflow() {
        let src = [
            255,255,255,255,255,255,255,
            255,255,255,255,255,255,255,
            255,255,255,  0,255,255,255,
            255,255,  0,  0,  0,255,255,
            255,255,255,  0,255,255,255,
            255,255,255,255,255,255,255,
            255,255,255,255,255,255,255 ];

        let expected = [
			255,255,251,  255,255,255,  255,255,255,  255,255,255,  255,255,255,  255,255,255,  255,255,251,
			255,255,255,  255,255,255,  255,255,255,  255,190,255,  255,255,255,  255,255,255,  255,255,255,
			255,255,255,  255,255,255,  255,111,174,  255,  0,111,  255,111,174,  255,255,255,  255,255,255,
			255,255,255,  255,190,255,  255,  0,111,  255,  0,  0,  255,  0,111,  255,190,255,  255,255,255,
			255,255,255,  255,255,255,  255,111,174,  255,  0,111,  255,111,174,  255,255,255,  255,255,255,
			255,255,255,  255,255,255,  255,255,255,  255,190,255,  255,255,255,  255,255,255,  255,255,255,
			255,255,251,  255,255,255,  255,255,255,  255,255,255,  255,255,255,  255,255,255,  255,255,251 ];

        const IMG_W: usize = 7;
        const IMG_H: usize = 7;
        let mut buf = [0u8; 3 * IMG_W * IMG_H];

        let res = debayer_u8(&mut Cursor::new(&src[..]), CFA::RGGB,
                &mut RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut buf));
        assert!(res.is_ok());
        assert_eq!(&buf[..], &expected[..]);
    }
}
