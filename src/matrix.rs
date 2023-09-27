use embedded_hal::digital::v2::{InputPin, OutputPin};
use crate::keyboard::*;

pub enum MatrixDirection {
	Col2Row,
	Row2Col,
}

// pub struct Matrix<'a, const I: usize, const O: usize> {
pub struct Matrix<I: InputPin<Error = ()>, O: OutputPin<Error = ()>> {
	// inputs: &'a [dyn InputPin<Error = ()>; I],
	// outputs: &'a [dyn OutputPin<Error = ()>; O],
	inputs: [I; 3],
	outputs: [O; 3],
	keymap: [KeyCode; 9],
	last_state: [Option<KeyEvent>; 9],
	direction: MatrixDirection,
}

// impl<const I: usize, const O: usize> Matrix<I, O> {
impl<I: InputPin<Error = ()>, O: OutputPin<Error = ()>> Matrix<I, O> {
	pub fn new(inputs: [I; 3], outputs: [O; 3], keymap: [KeyCode; 9], direction: MatrixDirection) -> Self {
		let last_state: [Option<KeyEvent>; 9] = Default::default();

		Matrix {
			inputs,
			outputs,
			keymap,
			last_state,
			direction,
		}
	}

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

impl<I: InputPin<Error = ()>, O: OutputPin<Error = ()>> KeyboardPoll for Matrix<I, O> {
	fn poll(&mut self) -> Option<KeyEvent> {
		let num_inputs = self.inputs.len();
		let num_outputs = self.outputs.len();

		for ii in 0..num_inputs {
			for oi in 0..num_outputs {
				self.write(ii, true);
				let state = self.read(oi);
				let last_state = &self.last_state[ii];

				// if state != last_state {
				// 	let key_code = self.keymap[ri * self.cols.len() + ci];
				// 	self.last_state[ri] = state;

				// 	if state == 1 {
				// 		return Some(KeyEvent::Pressed(key_code));
				// 	} else {
				// 		return Some(KeyEvent::Released(key_code));
				// 	}
				// }

				self.write(oi, false);
			}
		}

		None
	}
}
