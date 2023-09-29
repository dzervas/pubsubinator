use core::convert::Infallible;
use core::pin::Pin;

use alloc::{vec::Vec, boxed::Box};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use embassy_time::{Timer, Duration};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use futures::Future;
use defmt::*;
use crate::{reactor_event::*, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS};
use crate::reactor::{Polled, Producer};

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
	pub last_state: Vec<Vec<KeyEvent>>,
	pub event_buffer: Vec<KeyEvent>,
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
					row.push(KeyEvent::Released(c.clone()));
				}
				self.last_state.push(row);
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

impl<'a, I: InputObj, O: OutputObj> Polled for Matrix<'a, I, O> {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async move {
		let num_inputs = self.inputs.len();
		let num_outputs = self.outputs.len();

		for ii in 0..num_inputs {
			for oi in 0..num_outputs {
				self.write(ii, true);
				Timer::after(Duration::from_millis(1)).await;
				let state = self.read(oi);

				let (col, row) = match self.direction {
					MatrixDirection::Col2Row => (oi, ii),
					MatrixDirection::Row2Col => (ii, oi),
				};

				// TODO: Make this a bit more beautiful
				let new_state: KeyEvent = match &self.last_state[row][col] {
					KeyEvent::Pressed(code) => {
						if state {
							// KeyEvent::Held(code.clone())
							KeyEvent::Pressed(code.clone())
						} else {
							KeyEvent::Released(code.clone())
						}
					},
					KeyEvent::Released(code) => {
						if state {
							info!("Got a pressed event: {:?}", &code);
							KeyEvent::Pressed(code.clone())
						} else {
							KeyEvent::Released(code.clone())
						}
					},
					KeyEvent::Held(code) => {
						if !state {
							KeyEvent::Released(code.clone())
						} else {
							KeyEvent::Held(code.clone())
						}
					},
					KeyEvent::DoublePressed(code) => {
						if state {
							KeyEvent::Held(code.clone())
						} else {
							KeyEvent::Released(code.clone())
						}
					},
				};

				if new_state != self.last_state[row][col] {
					// self.event_buffer.push(new_state.clone());
					self.channel.publish(ReactorEvent::Key(new_state.clone())).await;
					self.last_state[row][col] = new_state;
				}

				self.write(oi, false);
			}
		}
		})
	}
}
