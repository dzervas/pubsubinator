use core::convert::Infallible;

use alloc::boxed::Box;
use alloc::vec::Vec;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use crate::keyboard::*;

pub enum MatrixDirection {
	Col2Row,
	Row2Col,
}

pub type InputPinArray = Vec<Box<dyn InputPin<Error = ()>>>;
pub type OutputPinArray = Vec<Box<dyn OutputPin<Error = ()>>>;
// pub type InputPinArray = &'static [Box<dyn InputPin<Error = ()>>];
// pub type OutputPinArray = &'static [Box<dyn OutputPin<Error = ()>>];

// TODO: Dynamic size
pub struct Matrix<I: InputPin<Error = Infallible>, O: OutputPin<Error = Infallible>> {
	// TODO: Use slices instead of vectors
	// inputs: Vec<Box<dyn InputPin<Error = ()>>>,
	// outputs: Vec<Box<dyn OutputPin<Error = ()>>>,
	// inputs: InputPinArray,
	// outputs: OutputPinArray,
	// TODO: Make these private and create platform-specific constructors
	pub inputs: Vec<I>,
	pub outputs: Vec<O>,
	pub keymap: Vec<KeyCode>,
	pub last_state: Vec<KeyEvent>,
	pub direction: MatrixDirection,
}

impl<I: InputPin<Error = Infallible>, O: OutputPin<Error = Infallible>> Matrix<I, O> {
	// pub fn new(inputs: Vec<Box<dyn InputPin<Error = ()>>>, outputs: Vec<Box<dyn OutputPin<Error = ()>>>, keymap: Vec<KeyCode>, direction: MatrixDirection) -> Self {
	// pub fn new(inputs: InputPinArray, outputs: OutputPinArray, keymap: Vec<KeyCode>, direction: MatrixDirection) -> Self {
	// pub fn new(inputs: InputPinArray, outputs: OutputPinArray, keymap: Vec<KeyCode>, direction: MatrixDirection) -> Self {
	// 	Matrix {
	// 		inputs,
	// 		outputs,
	// 		keymap,
	// 		last_state: Vec::new(),
	// 		direction,
	// 	}
	// }

	fn read(&self, index: usize) -> bool {
		self.inputs[index].is_high().unwrap()
	}

	fn write(&mut self, index: usize, value: bool) {
		if value {
			self.outputs[index].set_high().unwrap();
		} else {
			self.outputs[index].set_low().unwrap();
		}
	}
}

impl<I: InputPin<Error = Infallible>, O: OutputPin<Error = Infallible>> Matrix<I, O> {
	async fn poll(&mut self) {
		let num_inputs = self.inputs.len();
		let num_outputs = self.outputs.len();

		for ii in 0..num_inputs {
			for oi in 0..num_outputs {
				self.write(ii, true);
				// TODO: delay
				let state = self.read(oi);

				// TODO: Make last_state a 2 dimensional array
				match &self.last_state[ii] {
					KeyEvent::Pressed => {
						if state {
							self.last_state[ii] = KeyEvent::Held;
						} else {
							self.last_state[ii] = KeyEvent::Released;
						}
					},
					KeyEvent::Released => {
						if state {
							self.last_state[ii] = KeyEvent::Pressed;
						}
					},
					KeyEvent::Held => {
						if !state {
							self.last_state[ii] = KeyEvent::Released;
						}
					},
				}

				self.write(oi, false);
			}
		}
	}
}
