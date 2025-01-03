# hdrmerge-rs
An attempt at writing a tool for merging multiple (usually, but not necessarily, exposure-bracketed) raw images into a HDR image in 🦀 Rust.

Significantly inspired by [jcelaya's hdrmerge](https://github.com/jcelaya/hdrmerge).
Like hdrmerge, hdrmerge-rs operates on raw images in linear color space, without developing the raw photo, and outputs a floating-point HDR DNG image.

hdrmerge-rs uses [rawler](https://crates.io/crates/rawler) for parsing raw images and exporting DNG images.
 
