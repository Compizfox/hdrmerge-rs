use std::fs::File;
use std::io::BufWriter;
use rawler::dng::writer::DngWriter;
use rawler::dng::{CropMode, DngCompression, DngPhotometricConversion, DNG_VERSION_V1_4};
use rawler::{RawImage, RawImageData};
use clap::Parser;

#[derive(Parser)]
struct Args {
	/// Input files
	#[arg(required = true)]
	input_files: Vec<String>,
}

fn main() {
	let args: Args = Args::parse();

	let mut rawimages: Vec<RawImage> = args.input_files.iter().map(|filename| {
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

	let mut result: Vec<Vec<f32>> = vec![vec![]; a[0].len()];

	for (image_i, vec) in a.iter().enumerate() {
		println!("Adding image {}/{}...", image_i + 1, rawimages.len());

		let evs: [i32; 3] = [-2, 0, 2];
		for block_i in 0..vec.len() / 4 {
			if !is_saturated(vec, wl, width, block_i) {
				block(width, block_i)
					.iter()
					.for_each(|&i| {
						let corrected_val = vec[i].saturating_sub(bl) as f32
							/ 2f32.powi(evs[image_i]) + bl as f32;
						result[i].push(corrected_val)
					})
			}
		}
	}

	// Blend
	println!("Blending...");
	let blended = result.into_iter().map(|v| {
		v.iter().sum::<f32>() / v.len() as f32
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
fn block(width: usize, i: usize) -> [usize; 4] {
	let row_floored = 2 * i / width;
	let row_i = row_floored * width;
	[
		row_i         + 2 * i, row_i         + 2 * i + 1,
		row_i + width + 2 * i, row_i + width + 2 * i + 1,
	]
}

/// Returns true if one of the pixels in the 2x2 (CFA) block is saturated.
///
/// * `data`: Pixel array
/// * `wl`: White level indicating saturation
/// * `width`: Row width of the pixel array
/// * `block_i`: Block index
fn is_saturated(data: &Vec<u16>, wl: u32, width: usize, block_i: usize) -> bool {
	block(width, block_i)
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

		assert_eq!(crate::block(width, 0), [0, 1, 6, 7]);
		assert_eq!(crate::block(width, 1), [2, 3, 8, 9]);
		assert_eq!(crate::block(width, 3), [12, 13, 18, 19]);
	}
}
