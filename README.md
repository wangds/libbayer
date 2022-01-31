# `bayer` – Demoisaicing RAW Images From Bayer Sensors

[![Version][version-img]][version-url] [![Status][travis-ci-img]][travis-ci-url]

## About

Routines for demosaicing Bayer sensor (RAW) images.

Both 8-bit and 16-bit images are supported.

Several demosaicing algorithms are available.  See the `src/demosaic`
directory for a list and their individual descriptions.  Pixels on the
border of the image are retained by replicating or mirroring the data
in the neighbourhood.

The crate is written entirely in Rust.  C bindings to the underlying
algorithms are provided.

## Examples

An example program is provided in the `examples/` directory:

* `showbayer` – a simple Bayer file viewer.
* `writebayer` – converts an image to a raw Bayer image file.

To clone this repository, run:

```sh
git clone https://github.com/wangds/libbayer.git
```

Then build the library and run the example programs using Cargo.

```sh
cargo build --release --example showbayer
```

To display a Bayer file, run:

```sh
cargo run --release --example showbayer <width> <height> <depth> <example.raw>
```

Change the colour filter array (CFA) pattern and the demosaicing
algorithm from inside the example program.

## Basic Usage

Add `bayer` as a dependency to your project's `Cargo.toml`:

```toml
[dependencies]
bayer = "0.1"
```

Open a Bayer file from disk.

```rust
let mut file = File::open(Path::new("example.raw")).unwrap();
```

This raw data contains the red, green, or blue values at each pixel
only.  There is no additional header data, so the image width and
height, pixel depth, and CFA pattern must be provided from elsewhere.

Allocate the buffer to which we will decode the image.

```rust
let img_w = 320;
let img_h = 200;
let depth = bayer::RasterDepth::Depth8;
let bytes_per_pixel = 3;
let mut buf = vec![0; bytes_per_pixel * img_w * img_h];

let mut dst = bayer::RasterMut::new(img_w, img_h, depth, &mut buf);
```

Then run the demosaicing process:

```rust
let cfa = bayer::CFA::RGGB;
let alg = bayer::Demosaic::Linear;

bayer::run_demosaic(&mut file, bayer::BayerDepth:Depth8, cfa, alg, &mut dst);
```

Note that many cameras will capture 12-bits per pixel (channel), but
store the data as 16-bits per pixel.  These should be treated as
16-bits per pixel for the purposes of this library.

## Documentation

* [Documentation][documentation].

## Author

David Wang

[documentation]: https://docs.rs/bayer/
[travis-ci-img]: https://travis-ci.org/wangds/libbayer.svg?branch=master
[travis-ci-url]: https://travis-ci.org/wangds/libbayer
[version-img]: https://img.shields.io/crates/v/bayer.svg
[version-url]: https://crates.io/crates/bayer
