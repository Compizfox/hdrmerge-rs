mod util;

use std::fs::File;
use std::io::BufWriter;
use rawler::dng::writer::DngWriter;
use rawler::dng::{CropMode, DngCompression, DngPhotometricConversion, DNG_VERSION_V1_4};
use rawler::{RawImage, RawImageData};
use clap::Parser;
use rayon::prelude::*;

#[derive(Parser)]
struct Args {
	/// Input files
	#[arg(required = true)]
	input_files: Vec<String>,
}

#[derive(Clone, Copy)]
struct Sample {
	value: f32,
	weight: f32,
}

const evs: [i32; 3] = [-2, 0, 2];

fn main() {
	let args: Args = Args::parse();

	let rawimages: Vec<RawImage> = args.input_files.iter().map(|filename| {
		rawler::decode_file(filename).unwrap()
	}).collect();

	// Blend images
	let mut blended_rawimage = rawimages[0].clone();

	let bl = blended_rawimage.blacklevel.levels[0].as_f32() as u16;
	let wl = blended_rawimage.whitelevel.0[0];
	let width = blended_rawimage.width;

	let a = rawimages.iter().map(|rawimage| {
		match &rawimage.data {
			RawImageData::Integer(data) => data.clone(),
			_ => { panic!("Expected integer data") }
		}
	}).collect::<Vec<_>>();

	let n_pixels = a[0].len();

	let results: Vec<Vec<Sample>> = a.par_iter().enumerate().map(|(image_i, vec)| {
		println!("Adding image {}/{}...", image_i + 1, rawimages.len());

		let mut result: Vec<Sample> = vec![Sample { value: 0f32, weight: 0f32 }; n_pixels];

		(0..vec.len() / 4).into_iter().for_each(|block_i| {
			let block = block_to_indices(width, block_i);
			let is_saturated = is_saturated(vec, block, wl);
			let exp_factor = 2f32.powi(evs[image_i]);

			block
				.iter()
				.for_each(|&i| {
					result[i] = Sample {
						value: vec[i].saturating_sub(bl) as f32 / exp_factor + bl as f32,
						weight: if !is_saturated { exp_factor } else { 0f32 },
					};
				})
		});
		result
	}).collect();

	// Blend
	println!("Blending...");

	// Iterate over pixels
	let blended = (0..n_pixels)
		.into_par_iter()
		.map(|i| {
			// For each pixel, compute mean of non-zero pixels
			weighted_mean(results.iter().map(|col| col[i]))
		}).collect();

	blended_rawimage.data = RawImageData::Float(blended);

	// Save as DNG
	let output_stream = BufWriter::new(File::create("out.dng").unwrap());
	let mut dng_writer = DngWriter::new(output_stream, DNG_VERSION_V1_4).unwrap();
	let mut subframe_writer = dng_writer.subframe(0);
	subframe_writer.raw_image(&blended_rawimage, CropMode::Best, DngCompression::Uncompressed,
		DngPhotometricConversion::Original, 1);
	subframe_writer.finalize();

	dng_writer.close();
}


/// Returns the indices in flat pixel array of the i-th 2x2 (CFA) block.
///
/// The indices are numbered left-to-right, top-to-bottom, like:
/// ```
/// 1 2
/// 3 4
/// ```
///
/// * `width`: Row width of the pixel array
/// * `i`: Block index
fn block_to_indices(width: usize, i: usize) -> [usize; 4] {
	let row_floored = 2 * i / width;
	let row_i = row_floored * width;
	[
		row_i         + 2 * i, row_i         + 2 * i + 1,
		row_i + width + 2 * i, row_i + width + 2 * i + 1,
	]
}

/// Returns true if one of the pixels in the block is saturated.
///
/// * `data`: Pixel array
/// * `block`: Array of indices
/// * `wl`: White level indicating saturation
fn is_saturated(data: &Vec<u16>, block: [usize; 4], wl: u32) -> bool {
	block
		.iter()
		.map(|&i| data[i])
		.any(|x| x as u32 >= wl)
}

#[cfg(test)]
mod tests {
	#[test]
	fn block() {
		let array: [usize; 6 * 4] = core::array::from_fn(|i| i);
		let width = 6;

		assert_eq!(crate::block_to_indices(width, 0), [0, 1, 6, 7]);
		assert_eq!(crate::block_to_indices(width, 1), [2, 3, 8, 9]);
		assert_eq!(crate::block_to_indices(width, 3), [12, 13, 18, 19]);
	}
}

fn weighted_mean<I: IntoIterator<Item=Sample>>(xs: I) -> f32 {
	let mut sum = 0.0;
	let mut count = 0.0;

	for sample in xs {
		sum += sample.value * sample.weight;
		count += sample.weight;
	}

	sum / count
}
