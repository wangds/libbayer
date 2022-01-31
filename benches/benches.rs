//! Demosaicing algorithm benchmarks.

#![cfg_attr(feature = "bench", feature(test))]

#[cfg(all(feature = "bench", test))]
mod bench {
    use self::bayer::*;
    use std::io::Cursor;

    const IMG_W: usize = 128;
    const IMG_H: usize = 128;
    const SRC_U8: [u8; IMG_W * IMG_H] = [0u8; IMG_W * IMG_H];
    const SRC_U16: [u8; 2 * IMG_W * IMG_H] = [0u8; 2 * IMG_W * IMG_H];

    static mut BUF_U8: [u8; 3 * IMG_W * IMG_H] = [0u8; 3 * IMG_W * IMG_H];
    static mut BUF_U16: [u8; 6 * IMG_W * IMG_H] = [0u8; 6 * IMG_W * IMG_H];

    #[bench]
    fn bench_none_u8(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut BUF_U8) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U8[..]),
                BayerDepth::Depth8,
                CFA::RGGB,
                Demosaic::None,
                &mut dst,
            )
        });
    }

    #[bench]
    fn bench_none_u16(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth16, &mut BUF_U16) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U16[..]),
                BayerDepth::Depth16LE,
                CFA::RGGB,
                Demosaic::None,
                &mut dst,
            )
        });
    }

    #[bench]
    fn bench_nearest_neighbour_u8(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut BUF_U8) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U8[..]),
                BayerDepth::Depth8,
                CFA::RGGB,
                Demosaic::NearestNeighbour,
                &mut dst,
            )
        });
    }

    #[bench]
    fn bench_nearest_neighbour_u16(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth16, &mut BUF_U16) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U16[..]),
                BayerDepth::Depth16LE,
                CFA::RGGB,
                Demosaic::NearestNeighbour,
                &mut dst,
            )
        });
    }

    #[bench]
    fn bench_linear_u8(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut BUF_U8) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U8[..]),
                BayerDepth::Depth8,
                CFA::RGGB,
                Demosaic::Linear,
                &mut dst,
            )
        });
    }

    #[bench]
    fn bench_linear_u16(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth16, &mut BUF_U16) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U16[..]),
                BayerDepth::Depth16LE,
                CFA::RGGB,
                Demosaic::Linear,
                &mut dst,
            )
        });
    }

    #[bench]
    fn bench_cubic_u8(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth8, &mut BUF_U8) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U8[..]),
                BayerDepth::Depth8,
                CFA::RGGB,
                Demosaic::Cubic,
                &mut dst,
            )
        });
    }

    #[bench]
    fn bench_cubic_u16(b: &mut test::Bencher) {
        let mut dst = unsafe { RasterMut::new(IMG_W, IMG_H, RasterDepth::Depth16, &mut BUF_U16) };
        b.iter(|| {
            run_demosaic(
                &mut Cursor::new(&SRC_U16[..]),
                BayerDepth::Depth16LE,
                CFA::RGGB,
                Demosaic::Cubic,
                &mut dst,
            )
        });
    }
}
