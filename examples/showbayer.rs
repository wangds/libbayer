//! ShowBayer.
use bayer::*;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use std::{cmp::min, env, fs::File, path::Path, slice};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ImgDepth {
    Depth8,
    Depth12BE,
    Depth12LE,
    Depth16BE,
    Depth16LE,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    usage();
    if args.len() < 4 {
        return;
    }

    let bayer_w = args[0].parse::<usize>().unwrap();
    let bayer_h = args[1].parse::<usize>().unwrap();
    let depth = parse_depth(&args[2]);
    let files = &args[3..];

    let mut idx = 0;
    let mut cfa = CFA::BGGR;
    let mut alg = Demosaic::Linear;
    let mut old_idx = 1;
    let mut old_cfa = CFA::RGGB;
    let mut old_alg = Demosaic::None;

    // Initialise SDL window.
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    let window = video
        .window("ShowBayer", bayer_w as u32, bayer_h as u32)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, bayer_w as u32, bayer_h as u32)
        .unwrap();

    let bytes_per_pixel = bytes_per_pixel(raster_depth(depth));
    let mut buf = vec![0; bayer_w * bayer_h * bytes_per_pixel];

    read_file(
        &Path::new(&files[0]),
        bayer_w,
        bayer_h,
        depth,
        cfa,
        alg,
        &mut buf,
        &mut texture,
    );

    let mut redraw = true;
    'mainloop: loop {
        if let Some(e) = event_pump.wait_event_timeout(60) {
            match e {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'mainloop;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => {
                    cfa = CFA::BGGR;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => {
                    cfa = CFA::GBRG;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F3),
                    ..
                } => {
                    cfa = CFA::GRBG;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F4),
                    ..
                } => {
                    cfa = CFA::RGGB;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Num0),
                    ..
                } => {
                    alg = Demosaic::None;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => {
                    alg = Demosaic::NearestNeighbour;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => {
                    alg = Demosaic::Linear;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => {
                    alg = Demosaic::Cubic;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    idx = (idx + 1) % files.len();
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    if idx == 0 {
                        idx = files.len() - 1;
                    } else {
                        idx = idx - 1;
                    }
                }

                _ => (),
            }

            if idx != old_idx || cfa != old_cfa || alg != old_alg {
                redraw = true;
            }
        } else {
            redraw = true;
        }

        if redraw {
            if idx != old_idx || cfa != old_cfa || alg != old_alg {
                if old_idx != idx {
                    old_idx = idx;
                    println!("{}", files[idx]);
                }
                if old_cfa != cfa {
                    old_cfa = cfa;
                    print_cfa(cfa);
                }
                if old_alg != alg {
                    old_alg = alg;
                    print_alg(alg);
                }

                read_file(
                    &Path::new(&files[idx]),
                    bayer_w,
                    bayer_h,
                    depth,
                    cfa,
                    alg,
                    &mut buf,
                    &mut texture,
                );
            }

            present_to_screen(&mut canvas, &texture);
        }
    }
}

fn usage() {
    println!("usage: ShowBayer <width> <height> <depth> <filename> [filenames ...]");
    println!();
    println!("  depth     8, 12BE, 12LE, 16BE, 16LE");
    println!();
    println!("  <ESC>     Quit.");
    println!("  <left>    Go to previous image.");
    println!("  <right>   Go to previous image.");
    println!("  <space>   Go to next image.");
    println!();
    println!("  F1-F4     Change CFA pattern: BGGR, GBRG, GRBG, RGGB");
    println!("  0-3       Change demosaicing algorithm");
    println!();
}

fn parse_depth(s: &String) -> ImgDepth {
    let s = s.to_uppercase();
    if s == "8" {
        ImgDepth::Depth8
    } else if s == "12BE" {
        ImgDepth::Depth12BE
    } else if s == "12LE" {
        ImgDepth::Depth12LE
    } else if s == "16BE" {
        ImgDepth::Depth16BE
    } else if s == "16LE" {
        ImgDepth::Depth16LE
    } else {
        panic!("invalid depth");
    }
}

fn bayer_depth(depth: ImgDepth) -> BayerDepth {
    match depth {
        ImgDepth::Depth8 => BayerDepth::Depth8,
        ImgDepth::Depth12BE => BayerDepth::Depth16BE,
        ImgDepth::Depth12LE => BayerDepth::Depth16LE,
        ImgDepth::Depth16BE => BayerDepth::Depth16BE,
        ImgDepth::Depth16LE => BayerDepth::Depth16LE,
    }
}

fn raster_depth(depth: ImgDepth) -> RasterDepth {
    match depth {
        ImgDepth::Depth8 => RasterDepth::Depth8,
        ImgDepth::Depth12BE => RasterDepth::Depth16,
        ImgDepth::Depth12LE => RasterDepth::Depth16,
        ImgDepth::Depth16BE => RasterDepth::Depth16,
        ImgDepth::Depth16LE => RasterDepth::Depth16,
    }
}

fn bytes_per_pixel(depth: RasterDepth) -> usize {
    match depth {
        RasterDepth::Depth8 => 3,
        RasterDepth::Depth16 => 6,
    }
}

fn print_cfa(cfa: CFA) {
    let s = match cfa {
        CFA::BGGR => "BGGR",
        CFA::GBRG => "GBRG",
        CFA::GRBG => "GRBG",
        CFA::RGGB => "RGGB",
    };
    println!("CFA: {}", s);
}

fn print_alg(alg: Demosaic) {
    let s = match alg {
        Demosaic::None => "none",
        Demosaic::NearestNeighbour => "nearest neighbour",
        Demosaic::Linear => "linear",
        Demosaic::Cubic => "cubic",
    };
    println!("Demosaic: {}", s);
}

fn read_file(
    path: &Path,
    bayer_w: usize,
    bayer_h: usize,
    depth: ImgDepth,
    cfa: CFA,
    alg: Demosaic,
    buf: &mut [u8],
    texture: &mut sdl2::render::Texture,
) {
    let maybe_file = File::open(path);
    match maybe_file {
        Ok(mut f) => {
            let result = demosaic(
                &mut f,
                bayer_depth(depth),
                cfa,
                alg,
                &mut RasterMut::new(bayer_w, bayer_h, raster_depth(depth), buf),
            );
            match result {
                Ok(_) => (),
                Err(e) => {
                    println!("Error occurred - {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            println!("Error occurred - {}", e);
            return;
        }
    }

    render_to_texture(texture, bayer_w, bayer_h, depth, &buf);
}

fn render_to_texture(
    texture: &mut sdl2::render::Texture,
    w: usize,
    h: usize,
    depth: ImgDepth,
    buf: &[u8],
) {
    match raster_depth(depth) {
        RasterDepth::Depth8 => {
            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..h {
                        let src_offset = (3 * w) * y;
                        let dst_offset = pitch * y;

                        for i in 0..3 * w {
                            buffer[dst_offset + i] = buf[src_offset + i];
                        }
                    }
                })
                .unwrap();
        }

        RasterDepth::Depth16 => {
            let shr = if depth == ImgDepth::Depth12BE || depth == ImgDepth::Depth12LE {
                4
            } else {
                8
            };
            let buf = unsafe { slice::from_raw_parts(buf.as_ptr() as *const u16, buf.len() / 2) };

            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..h {
                        let src_offset = (3 * w) * y;
                        let dst_offset = pitch * y;

                        for i in 0..3 * w {
                            // shr = 8 for u16 to u8, or
                            // shr = 4 for u12 to u8.
                            let v = buf[src_offset + i] >> shr;
                            buffer[dst_offset + i] = min(v, 255) as u8;
                        }
                    }
                })
                .unwrap();
        }
    }
}

fn present_to_screen(canvas: &mut sdl2::render::WindowCanvas, texture: &sdl2::render::Texture) {
    canvas.clear();
    let _ = canvas.copy(&texture, None, None);
    canvas.present();
}
