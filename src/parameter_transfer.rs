
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};

const USIZE_BITS: usize = size_of::<usize>() * 8;

fn word_and_bit(index: usize) -> (usize, usize) {
	(index / USIZE_BITS, 1usize << (index & (USIZE_BITS - 1)))
}

#[derive(Default)]
pub struct ParameterTransfer {
	values: Vec<AtomicUsize>,
	changed: Vec<AtomicUsize>,
}

impl ParameterTransfer {
	pub fn new(parameter_count: usize) -> Self {
		let bit_words = (parameter_count + USIZE_BITS - 1) / USIZE_BITS;
		ParameterTransfer {
			values: (0..parameter_count).map(|_| AtomicUsize::new(0)).collect(),
			changed: (0..bit_words).map(|_| AtomicUsize::new(0)).collect(),
		}
	}

	pub fn set_parameter(&self, index: usize, value: f32) {
		let (word, bit) = word_and_bit(index);
		self.values[index].store(value.to_bits() as usize, Ordering::Relaxed);
		self.changed[word].fetch_or(bit, Ordering::AcqRel);
	}

	pub fn get_parameter(&self, index: usize) -> f32 {
		f32::from_bits(self.values[index].load(Ordering::Relaxed) as u32)
	}

	pub fn iterate<'pt>(&'pt self, acquire: bool) -> ParameterTransferIterator<'pt> {
		ParameterTransferIterator {
			pt: self,
			word: 0,
			bit: 1,
			acquire,
		}
	}
}

pub struct ParameterTransferIterator<'pt> {
	pt: &'pt ParameterTransfer,
	word: usize,
	bit: usize,
	acquire: bool,
}

impl<'pt> Iterator for ParameterTransferIterator<'pt> {
	type Item = (usize, f32);

	fn next(&mut self) -> Option<(usize, f32)> {
		let bits = loop {
			if self.word == self.pt.changed.len() {
				return None
			}
			let bits = self.pt.changed[self.word].load(Ordering::Acquire) & self.bit.wrapping_neg();
			if bits != 0 { break bits; }
			self.word += 1;
			self.bit = 1;
		};

		let bit_index = bits.trailing_zeros() as usize;
		let bit = 1usize << bit_index;
		let index = self.word * USIZE_BITS + bit_index;

		if self.acquire {
			self.pt.changed[self.word].fetch_and(!bit, Ordering::AcqRel);
		}

		let next_bit = bit << 1;
		if next_bit == 0 {
			self.word += 1;
			self.bit = 1;
		} else {
			self.bit = next_bit;
		}

		Some((index, self.pt.get_parameter(index)))
	}
}
