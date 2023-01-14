//! Foreign function interface.

use libc::{c_uchar, c_uint, size_t};
use std::io::{Cursor, Read};
use std::mem;
use std::ptr;
use std::slice;

use crate::demosaic;
use crate::{BayerDepth, BayerError, BayerResult, RasterDepth, RasterMut, CFA};

/// Dummy opaque structure, equivalent to [`RasterMut`].
pub struct CRasterMut;

// Print with "file:line - " prefix, for more informative error messages.
macro_rules! printerrorln {
    ($e:expr) => {{
        println!("{}:{} - {}", file!(), line!(), $e);
    }};
    ($fmt:expr, $arg:tt) => {{
        print!("{}:{} - ", file!(), line!());
        println!($fmt, $arg);
    }};
}

unsafe fn transmute_raster_mut<'a>(dst: *mut CRasterMut) -> &'a mut RasterMut<'a> {
    let ptr: *mut RasterMut = mem::transmute(dst);
    &mut *ptr
}

#[allow(clippy::too_many_arguments)]
fn run_demosaic<F>(
    file: &'static str,
    line: u32,
    run: F,
    src: *const c_uchar,
    src_len: size_t,
    depth: c_uint,
    be: c_uint,
    cfa: c_uint,
    dst: *mut CRasterMut,
) -> c_uint
where
    F: FnOnce(&mut dyn Read, BayerDepth, CFA, &mut RasterMut) -> BayerResult<()>,
{
    if src.is_null() || dst.is_null() {
        println!("{file} {line} - bad input parameters");
        return 1;
    }

    let depth = match (depth, be) {
        (8, _) => BayerDepth::Depth8,
        (16, 0) => BayerDepth::Depth16LE,
        (16, _) => BayerDepth::Depth16BE,
        _ => {
            println!("{file} {line} - invalid depth");
            return 2;
        }
    };

    let cfa = match cfa {
        0 => CFA::BGGR,
        1 => CFA::GBRG,
        2 => CFA::GRBG,
        3 => CFA::RGGB,
        _ => {
            println!("{file} {line} - invalid cfa");
            return 1;
        }
    };

    let src_slice = unsafe { slice::from_raw_parts(src, src_len) };
    let dst_raster = unsafe { transmute_raster_mut(dst) };

    match run(&mut Cursor::new(src_slice), depth, cfa, dst_raster) {
        Ok(_) => 0,
        Err(BayerError::WrongResolution) => 2,
        Err(BayerError::WrongDepth) => 3,
        Err(_) => 1,
    }
}

/* -------------------------------------------------------------- */
/* Demosaicing algorithms */
/* -------------------------------------------------------------- */

/// Demosaicing without any interpolation.
///
/// Before using this, allocate a [`CRasterMut`] for `dst` by calling
/// [`bayerrs_raster_mut_alloc()`].
///
/// This modifies the [`buf`] you passed in the aforementioned call.
#[no_mangle]
pub extern "C" fn bayerrs_demosaic_none(
    src: *const c_uchar,
    src_len: size_t,
    depth: c_uint,
    be: c_uint,
    cfa: c_uint,
    dst: *mut CRasterMut,
) -> c_uint {
    run_demosaic(
        file!(),
        line!(),
        demosaic::none::run,
        src,
        src_len,
        depth,
        be,
        cfa,
        dst,
    )
}

/// Demosaicing using nearest neighbour interpolation.
#[no_mangle]
pub extern "C" fn bayerrs_demosaic_nearest_neighbour(
    src: *const c_uchar,
    src_len: size_t,
    depth: c_uint,
    be: c_uint,
    cfa: c_uint,
    dst: *mut CRasterMut,
) -> c_uint {
    run_demosaic(
        file!(),
        line!(),
        demosaic::nearestneighbour::run,
        src,
        src_len,
        depth,
        be,
        cfa,
        dst,
    )
}

/// Demosaicing using linear interpolation.
#[no_mangle]
pub extern "C" fn bayerrs_demosaic_linear(
    src: *const c_uchar,
    src_len: size_t,
    depth: c_uint,
    be: c_uint,
    cfa: c_uint,
    dst: *mut CRasterMut,
) -> c_uint {
    run_demosaic(
        file!(),
        line!(),
        demosaic::linear::run,
        src,
        src_len,
        depth,
        be,
        cfa,
        dst,
    )
}

/// Demosaicing using cubic interpolation.
#[no_mangle]
pub extern "C" fn bayerrs_demosaic_cubic(
    src: *const c_uchar,
    src_len: size_t,
    depth: c_uint,
    be: c_uint,
    cfa: c_uint,
    dst: *mut CRasterMut,
) -> c_uint {
    run_demosaic(
        file!(),
        line!(),
        demosaic::cubic::run,
        src,
        src_len,
        depth,
        be,
        cfa,
        dst,
    )
}

/* -------------------------------------------------------------- */
/* Raster */
/* -------------------------------------------------------------- */

/// Allocate a new [`CRasterMut`].
///
/// # Safety
///
/// The `buf` point to a region of memory with at least `buf_len` size and big
/// enough to hold the geometry described by the first six parameters.
///
/// Otherwise calling any of the `bayerrs_demosaic...()` functions on the
/// resulting [`CRasterMut`] pointer will cause undefined behaviour.
///
/// When you are done with the returned `CRasterMut`, pass it to
/// [`bayerrs_raster_mut_free()`] to deallocate the memoriy it points to.
/// Otherwise calling this function will leak.
#[no_mangle]
pub unsafe extern "C" fn bayerrs_raster_mut_alloc(
    x: size_t,
    y: size_t,
    w: size_t,
    h: size_t,
    stride: size_t,
    depth: c_uint,
    buf: *mut c_uchar,
    buf_len: size_t,
) -> *mut CRasterMut {
    if buf.is_null() {
        printerrorln!("bad input parameters");
        return ptr::null_mut();
    }

    let depth = match depth {
        8 => RasterDepth::Depth8,
        16 => RasterDepth::Depth16,
        _ => {
            printerrorln!("bad input parameters");
            return ptr::null_mut();
        }
    };

    let buf_slice = slice::from_raw_parts_mut(buf, buf_len);
    let raster = RasterMut::with_offset(x, y, w, h, stride, depth, buf_slice);
    let rptr = Box::into_raw(Box::new(raster));
    let cptr: *mut CRasterMut = mem::transmute(rptr);
    cptr
}

/// Free a previously allocated [`CRasterMut`].
///
/// # Safety
///
/// This will do nothing if `raster` is a null pointer.
///
/// The `raster` pointer was must have been initialized from calling
/// [`bayerrs_raster_mut_alloc()`] initially. Otherwise calling this function
/// will cause undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn bayerrs_raster_mut_free(raster: *mut CRasterMut) {
    if raster.is_null() {
        return;
    }

    let rptr: *mut RasterMut = mem::transmute(raster);
    let _raster = Box::from_raw(rptr);
}
