mod util;
mod sample;
mod cfa;

use std::fs::File;
use std::io::BufWriter;

use clap::Parser;
use rayon::prelude::*;
use rawler::dng::writer::DngWriter;
use rawler::dng::{CropMode, DngCompression, DngPhotometricConversion, DNG_VERSION_V1_4};
use rawler::{RawImage, RawImageData};
use rawler::decoders::RawMetadata;
use rawler::formats::tiff::SRational;

use sample::{Sample, weighted_mean};
use cfa::{block_to_indices, is_saturated};

#[derive(Parser)]
struct Args {
	/// Input files
	#[arg(required = true)]
	input_files: Vec<String>,
}

fn main() {
	let args: Args = Args::parse();

	let (rawimages, metadatas): (Vec<RawImage>, Vec<RawMetadata>) = args.input_files.iter().map(|filename| {
		let rawfile = rawler::rawsource::RawSource::new(filename.as_ref()).unwrap();
		let decoder = rawler::get_decoder(&rawfile).unwrap();
		let raw_params = rawler::decoders::RawDecodeParams { image_index: 0 };
		let rawimage = decoder.raw_image(&rawfile, &raw_params, false).unwrap();
		let metadata = decoder.raw_metadata(&rawfile, &raw_params).unwrap();

		(rawimage, metadata)
	}).unzip();

	// Get exposure values
	let evs: Vec<SRational> = metadatas
		.iter()
		.map(|metadata| metadata.exif.exposure_bias.unwrap())
		.collect();

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
			let exp_factor = 2f32.powf(evs[image_i].to_f32());

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

trait SRationalTo {
	fn to_f32(self) -> f32;
}

impl SRationalTo for SRational {
	fn to_f32(self) -> f32 {
		self.n as f32 / self.d as f32
	}
}