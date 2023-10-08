use core::convert::Infallible;
use core::pin::Pin;

use alloc::{vec::Vec, boxed::Box};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use futures::Future;
use crate::{PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS};
use reactor::reactor_event::*;
use reactor::{Polled, RPublisher};

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
	pub last_state: Vec<Vec<bool>>,
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

impl<'a, I: InputObj, O: OutputObj> RPublisher for Matrix<'a, I, O> {
	fn setup(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			let (cols, rows) = match self.direction {
				MatrixDirection::Col2Row => (self.outputs.len(), self.inputs.len()),
				MatrixDirection::Row2Col => (self.outputs.len(), self.inputs.len()),
			};

			for _ in 0..rows {
				self.last_state.push(Vec::new());

				for _ in 0..cols {
					self.last_state.last_mut().unwrap().push(false);
				}
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

				for ii in 0..num_inputs {
					let state = self.read(ii);

					let (col, row) = match self.direction {
						MatrixDirection::Col2Row => (oi, ii),
						MatrixDirection::Row2Col => (ii, oi),
					};

					if state != self.last_state[row][col] {
						self.last_state[row][col] = state;

						let event = ReactorEvent::HardwareMappedBool(state, row, col);
						event_buffer.push(event);
					}
				}

				self.write(oi, false);
			}

			for event in event_buffer {
				self.channel.publish(event).await;
			}
		})
	}
}
