
mod parameter_transfer;

use std::f32;
use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, HostCallback, Info, Plugin, PluginParameters};
use vst::plugin_main;

use crate::parameter_transfer::*;

const PARAMETER_COUNT: usize = 100;
const BASE_FREQUENCY: f32 = 5.0;
const FILTER_FACTOR: f32 = 0.01;
const TWO_PI: f32 = 2.0 * f32::consts::PI;

#[derive(Default)]
struct MyPlugin {
	params: Arc<MyPluginParameters>,
	states: Vec<Smoothed>,
	sample_rate: f32,
	phase: f32,
}

#[derive(Default)]
struct MyPluginParameters {
	#[allow(dead_code)]
	host: HostCallback,
	transfer: ParameterTransfer,
}

#[derive(Clone, Default)]
struct Smoothed {
	state: f32,
	target: f32,
}

impl Smoothed {
	fn set(&mut self, value: f32) {
		self.target = value;
	}

	fn get(&mut self) -> f32 {
		self.state += (self.target - self.state) * FILTER_FACTOR;
		self.state
	}
}

impl Plugin for MyPlugin {
	fn new(host: HostCallback) -> Self {
		MyPlugin {
			params: Arc::new(MyPluginParameters {
				host,
				transfer: ParameterTransfer::new(PARAMETER_COUNT),
			}),
			states: vec![Smoothed::default(); PARAMETER_COUNT],
			sample_rate: 44100.0,
			phase: 0.0,
		}
	}

	fn get_info(&self) -> Info {
		Info {
			parameters: PARAMETER_COUNT as i32,
			inputs: 0,
			outputs: 2,
			category: Category::Synth,
			f64_precision: false,

			name: "transfer_and_smooth".to_string(),
			vendor: "Loonies".to_string(),
			unique_id: 0x500007,
			version: 100,

			.. Info::default()
		}
	}

	fn get_parameter_object(&mut self) -> Arc<PluginParameters> {
		Arc::clone(&self.params) as Arc<PluginParameters>
	}

	fn set_sample_rate(&mut self, sample_rate: f32) {
		self.sample_rate = sample_rate;
	}

	fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
		// Update filter state of all changed parameters.
		for (p, value) in self.params.transfer.iterate(true) {
			self.states[p].set(value);
		}

		// Dummy synth adding together a bunch of sines.
		let samples = buffer.samples();
		let mut outputs = buffer.split().1;
		for i in 0..samples {
			let mut sum = 0.0;
			for p in 0..PARAMETER_COUNT {
				let amp = self.states[p].get();
				if amp != 0.0 {
					sum += (self.phase * p as f32 * TWO_PI).sin() * amp;
				}
			}
			outputs[0][i] = sum;
			outputs[1][i] = sum;
			self.phase = (self.phase + BASE_FREQUENCY / self.sample_rate).fract();
		}
	}
}

impl PluginParameters for MyPluginParameters {
	fn set_parameter(&self, index: i32, value: f32) {
		self.transfer.set_parameter(index as usize, value);
	}

	fn get_parameter(&self, index: i32) -> f32 {
		self.transfer.get_parameter(index as usize)
	}
}

plugin_main!(MyPlugin);
