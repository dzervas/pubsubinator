use core::convert::Infallible;
use core::pin::Pin;

use alloc::{vec::Vec, boxed::Box};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use futures::Future;
use defmt::*;
use crate::reactor_event::*;
use crate::reactor::{Polled, Producer};

pub enum MatrixDirection {
	Col2Row,
	Row2Col,
}

pub trait InputObj = InputPin<Error = Infallible>;
pub trait OutputObj = OutputPin<Error = Infallible>;
// pub type InputPinArray = &'static [Box<dyn InputPin<Error = ()>>];
// pub type OutputPinArray = &'static [Box<dyn OutputPin<Error = ()>>];

// TODO: Dynamic size
pub struct Matrix<I: InputObj, O: OutputObj> {
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
	pub event_buffer: Vec<KeyEvent>,
	pub direction: MatrixDirection,
}

impl<I: InputObj, O: OutputObj> Matrix<I, O> {
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

impl<I: InputObj, O: OutputObj> Producer for Matrix<I, O> {
	fn setup(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			for k in self.keymap.iter() {
				self.last_state.push(KeyEvent::Released(*k));
			}
		})
	}

	fn get_state(&mut self) -> Pin<Box<dyn Future<Output = Option<ReactorEvent>> + '_>> {
		Box::pin(async {
			self.poll().await;

			if self.event_buffer.len() == 0 {
				return None
			}

			Some(ReactorEvent::Key(self.event_buffer.remove(0)))
		})
	}
}

impl<I: InputObj, O: OutputObj> Polled for Matrix<I, O> {
	async fn poll(&mut self) {
		let num_inputs = self.inputs.len();
		let num_outputs = self.outputs.len();

		for ii in 0..num_inputs {
			for oi in 0..num_outputs {
				self.write(ii, true);
				// TODO: delay
				let state = self.read(oi);

				if self.last_state.len() <= ii {
					self.last_state.push(KeyEvent::Released(self.keymap[ii]));
				}

				// TODO: Make last_state a 2 dimensional array
				match &self.last_state[ii] {
					KeyEvent::Pressed(code) => {
						if state {
							self.last_state[ii] = KeyEvent::Held(code.clone());
						} else {
							self.last_state[ii] = KeyEvent::Released(code.clone());
						}
					},
					KeyEvent::Released(code) => {
						if state {
							info!("Got a pressed event: {:?}", &code);
							self.last_state[ii] = KeyEvent::Held(code.clone());
						}
					},
					KeyEvent::Held(code) => {
						if !state {
							self.last_state[ii] = KeyEvent::Held(code.clone());
						}
					},
					KeyEvent::DoublePressed(code) => {
						if state {
							self.last_state[ii] = KeyEvent::Held(code.clone());
						}
					},
				}

				// self.write(oi, false);
			}
		}
	}
}
