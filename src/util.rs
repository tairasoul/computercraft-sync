#[cfg(test)]
pub fn randstring(length: usize) -> String {
	use rand::Rng;
	rand::rng()
		.sample_iter(&rand::distr::Alphanumeric)
		.take(length)
		.map(char::from)
		.collect()
}