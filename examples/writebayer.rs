//! WriteBayer.

extern crate bayer;
extern crate flic;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path,PathBuf};
use sdl2::image::LoadSurface;
use sdl2::surface::Surface;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    usage();
    if args.len() < 2 {
        return;
    }

    sdl2::image::init(sdl2::image::INIT_JPG | sdl2::image::INIT_PNG | sdl2::image::INIT_TIF)
        .unwrap();

    let cfa = parse_cfa(&args[0]);
    let files = &args[1..];

    for f in files {
        let src = Path::new(f);
        if !src.exists() {
            println!("{} does not exist", f);
            continue;
        }

        let mut dst = PathBuf::from(f);
        dst.set_extension("raw");
        if dst.exists() {
            println!("{} already exists", dst.display());
            continue;
        }

        if let Ok(mut flic) = flic::FlicFile::open(&src) {
            let w = flic.width() as usize;
            let h = flic.height() as usize;
            let mut buf = vec![0; w * h];
            let mut pal = vec![0; 3 * 256];

            if flic.read_next_frame(
                    &mut flic::RasterMut::new(w, h, &mut buf, &mut pal)).is_err() {
                continue;
            }

            write_mosaic_pal(&dst, &buf, &pal, w, h, cfa);
            continue;
        }

        if let Ok(surface) = Surface::from_file(&src) {
            let w = surface.width() as usize;
            let h = surface.height() as usize;
            surface.with_lock(|s| write_mosaic_rgba(&dst, s, w, h, cfa));
            continue;
        }
    }
}

fn usage() {
    println!("usage: WriteBayer <cfa> <filename> [filenames ...]");
    println!();
    println!("  cfa       BGGR, GBRG, GRBG, RGGB");
    println!();
}

fn parse_cfa(s: &String) -> bayer::CFA {
    let s = s.to_uppercase();
    if s == "BGGR" {
        bayer::CFA::BGGR
    } else if s == "GBRG" {
        bayer::CFA::GBRG
    } else if s == "GRBG" {
        bayer::CFA::GRBG
    } else if s == "RGGB" {
        bayer::CFA::RGGB
    } else {
        panic!("invalid cfa");
    }
}

fn write_mosaic_rgba(dst: &PathBuf,
        s: &[u8], w: usize, h: usize, cfa: bayer::CFA) {
    let mut v = Vec::with_capacity(w * h);
    let mut cfa_y = cfa;

    for y in 0..h {
        let mut cfa_x = cfa_y;

        for x in 0..w {
            let c = match cfa_x {
                bayer::CFA::BGGR =>
                    s[4 * w * y + 4 * x + 2],
                bayer::CFA::GBRG | bayer::CFA::GRBG =>
                    s[4 * w * y + 4 * x + 1],
                bayer::CFA::RGGB =>
                    s[4 * w * y + 4 * x + 0],
            };

            v.push(c);
            cfa_x = cfa_x.next_x();
        }

        cfa_y = cfa_y.next_y();
    }

    if let Ok(mut fp) = File::create(&dst) {
        println!("writing {} [{}x{}]", dst.display(), w, h);
        let _ = fp.write_all(&v[..]);
    }
}

fn write_mosaic_pal(dst: &PathBuf,
        buf: &[u8], pal: &[u8], w: usize, h: usize, cfa: bayer::CFA) {
    let mut v = Vec::with_capacity(w * h);
    let mut cfa_y = cfa;

    for y in 0..h {
        let mut cfa_x = cfa_y;

        for x in 0..w {
            let c = match cfa_x {
                bayer::CFA::BGGR =>
                    pal[3 * buf[w * y + x] as usize + 2],
                bayer::CFA::GBRG | bayer::CFA::GRBG =>
                    pal[3 * buf[w * y + x] as usize + 1],
                bayer::CFA::RGGB =>
                    pal[3 * buf[w * y + x] as usize + 0],
            };

            v.push(c);
            cfa_x = cfa_x.next_x();
        }

        cfa_y = cfa_y.next_y();
    }

    if let Ok(mut fp) = File::create(&dst) {
        println!("writing {} [{}x{}]", dst.display(), w, h);
        let _ = fp.write_all(&v[..]);
    }
}
