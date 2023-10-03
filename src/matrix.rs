use core::convert::Infallible;
use core::pin::Pin;

use alloc::{vec::Vec, boxed::Box};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use futures::Future;
use defmt::*;
use crate::{reactor_event::*, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS};
use crate::reactor::{Polled, Producer};

pub const MATRIX_PERIOD: u64 = 2;
pub const DEBOUNCE_CYCLES: u8 = 3;
// pub const HOLD_CYCLES: u8 = 200;

pub enum MatrixDirection {
	Col2Row,
	Row2Col,
}

pub trait InputObj = InputPin<Error = Infallible>;
pub trait OutputObj = OutputPin<Error = Infallible>;

// TODO: Dynamic size
pub struct Matrix<'a, I: InputObj, O: OutputObj> {
	// TODO: Use slices instead of vectors
	// TODO: Make these private and create platform-specific constructors
	pub inputs: Vec<I>,
	pub outputs: Vec<O>,
	pub keymap: Vec<Vec<KeyCode>>,
	pub last_state: Vec<Vec<(KeyEvent, u8)>>,
	pub direction: MatrixDirection,
	pub channel: Publisher<'a, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>
}

impl<'a, I: InputObj, O: OutputObj> Matrix<'a, I, O> {
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

impl<'a, I: InputObj, O: OutputObj> Producer for Matrix<'a, I, O> {
	fn setup(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			for r in self.keymap.iter() {
				let mut row = Vec::new();
				for c in r.iter() {
					row.push((KeyEvent::Released(c.clone()), 0));
				}
				self.last_state.push(row);
			}
		})
	}
}

impl<'a, I: InputObj, O: OutputObj> Polled for Matrix<'a, I, O> {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async move {
			let mut event_buffer = Vec::new();
			let num_inputs = self.inputs.len();
			let num_outputs = self.outputs.len();

			for oi in 0..num_outputs {
				self.write(oi, true);
				// Timer::after(Duration::from_micros(10)).await;

				for ii in 0..num_inputs {
					let state = self.read(ii);

					let (col, row) = match self.direction {
						MatrixDirection::Col2Row => (oi, ii),
						MatrixDirection::Row2Col => (ii, oi),
					};

					self.last_state[row][col].1 += 1;

					// TODO: Make this a bit more beautiful
					let new_state: KeyEvent = match &self.last_state[row][col].0 {
						KeyEvent::Released(code) => {
							if state && self.last_state[row][col].1 >= DEBOUNCE_CYCLES {
								info!("Got a pressed event: {:?}", &code);
								KeyEvent::Pressed(code.clone())
							} else {
								KeyEvent::Released(code.clone())
							}
						},
						KeyEvent::Pressed(code) => {
							if state {
								KeyEvent::Pressed(code.clone())
							} else {
								info!("Got a released event: {:?}", &code);
								KeyEvent::Released(code.clone())
							}
						},
					};

					if self.last_state[row][col].1 == 255 {
						match new_state {
							KeyEvent::Released(_) => self.last_state[row][col].1 = 0,
							KeyEvent::Pressed(_) => self.last_state[row][col].1 = DEBOUNCE_CYCLES + 1,
						};
					}

					if new_state != self.last_state[row][col].0 {
						// self.channel.publish_immediate(ReactorEvent::Key(new_state.clone()));
						event_buffer.push(ReactorEvent::Key(new_state.clone()));
						self.last_state[row][col].0 = new_state;
					}
				}

				self.write(oi, false);
			}

			// event_buffer.reverse();
			for e in event_buffer.iter() {
				self.channel.publish_immediate(e.clone());
			}
		})
	}
}
