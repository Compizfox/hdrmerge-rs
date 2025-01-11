pub struct DynMultiZip<I: Iterator>(pub Vec<I>);

impl<I: Iterator> Iterator for DynMultiZip<I> {
	type Item = Vec<I::Item>;
	fn next(&mut self) -> Option<Self::Item> {
		self.0
			.iter_mut()
			.map(|iter| iter.next())
			.collect::<Option<Vec<_>>>()
	}
}
