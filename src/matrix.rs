use core::convert::Infallible;
use core::pin::Pin;

use crate::{PUBSUB_CAPACITY, PUBSUB_PUBLISHERS, PUBSUB_SUBSCRIBERS};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use embedded_hal::digital::{InputPin, OutputPin};
use futures::Future;
use reactor::reactor_event::*;
use reactor::{Polled, RPublisher};
use strum::EnumString;

pub const MATRIX_PERIOD: u64 = 2;
// pub const HOLD_CYCLES: u8 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum MatrixDirection {
	Col2Row,
	Row2Col,
}

// TODO: Dynamic size
pub struct Matrix<'a, I: InputPin<Error = Infallible>, O: OutputPin<Error = Infallible>> {
	// TODO: Use slices instead of vectors
	// TODO: Make these private and create platform-specific constructors
	inputs: Vec<I>,
	outputs: Vec<O>,
	last_state: Vec<Vec<bool>>,
	direction: MatrixDirection,
	channel:
		Publisher<'a, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>,
}

impl<'a, I: InputPin<Error = Infallible>, O: OutputPin<Error = Infallible>> Matrix<'a, I, O> {
	pub fn new(inputs: Vec<I>, outputs: Vec<O>, direction: MatrixDirection) -> Self {
		let last_state = vec![vec![false; inputs.len()]; outputs.len()];

		Self {
			inputs,
			outputs,
			last_state,
			direction,
			channel: crate::CHANNEL.publisher().unwrap(),
		}
	}

	fn read(&mut self, index: usize) -> bool {
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

impl<'a, I: InputPin<Error = Infallible>, O: OutputPin<Error = Infallible>> RPublisher for Matrix<'a, I, O> {}

impl<'a, I: InputPin<Error = Infallible>, O: OutputPin<Error = Infallible>> Polled for Matrix<'a, I, O> {
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
