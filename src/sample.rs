#[derive(Clone, Copy)]
pub struct Sample {
	pub value: f32,
	pub weight: f32,
}

pub fn weighted_mean<I: IntoIterator<Item=Sample>>(xs: I) -> f32 {
	let mut sum = 0.0;
	let mut count = 0.0;

	for sample in xs {
		sum += sample.value * sample.weight;
		count += sample.weight;
	}

	sum / count
}