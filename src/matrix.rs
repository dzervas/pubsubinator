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
// pub const HOLD_CYCLES: u8 = 200;

pub enum MatrixDirection {
	Col2Row,
	Row2Col,
}

pub trait InputObj = InputPin<Error = Infallible>;
pub trait OutputObj = OutputPin<Error = Infallible>;

// TODO: Dynamic size
pub struct Matrix<'a, I: InputObj, O: OutputObj, const IC: usize, const OC: usize> {
	// TODO: Use slices instead of vectors
	// TODO: Make these private and create platform-specific constructors
	pub inputs: [I; IC],
	pub outputs: [O; OC],
	pub last_state: [[bool; OC]; IC],
	pub direction: MatrixDirection,
	pub channel: Publisher<'a, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>
}

impl<'a, I: InputObj, O: OutputObj, const IC: usize, const OC: usize> Matrix<'a, I, O, IC, OC> {
	pub fn new(inputs: [I; IC], outputs: [O; OC], direction: MatrixDirection) -> Self {
		let last_state = [[false; OC]; IC];

		Self {
			inputs,
			outputs,
			last_state,
			direction,
			channel: crate::CHANNEL.publisher().unwrap()
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

impl<'a, I: InputObj, O: OutputObj, const IC: usize, const OC: usize> RPublisher for Matrix<'a, I, O, IC, OC> {
}

impl<'a, I: InputObj, O: OutputObj, const IC: usize, const OC: usize> Polled for Matrix<'a, I, O, IC, OC> {
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
