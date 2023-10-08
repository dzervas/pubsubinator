use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use defmt::*;

use embassy_nrf::saadc::Saadc;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;

// TODO: Use a generics instead of nrf-specifics
use embassy_nrf::Peripheral;
use embassy_nrf::saadc::Input;
use embassy_nrf::peripherals::SAADC;

use reactor::reactor_event::*;
use reactor::{Polled, RPublisher};
use crate::{PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS, Irqs};

pub struct Analog<'a, const N: usize> {
	input: Saadc<'a, N>,
	last_state: [i16; N],
	channel: Publisher<'a, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>,
}

impl<'a, const N: usize> Analog<'a, N> {
	pub fn new(p_saadc: SAADC, input: [&'static mut impl Peripheral<P = impl Input>; N]) -> Self {
		let config = embassy_nrf::saadc::Config::default();
		let mut input_iter = input.into_iter();
		let channel_config = [(); N].map(|_| embassy_nrf::saadc::ChannelConfig::single_ended(input_iter.next().unwrap()));
		let saadc: Saadc<'a, N> = Saadc::new(p_saadc, Irqs, config, channel_config);

		Self {
			input: saadc,
			last_state: [0; N],
			channel: crate::CHANNEL.publisher().unwrap(),
		}
	}
}

impl<'a, const N: usize> RPublisher for Analog<'a, N> {}

impl<'a, const N: usize> Polled for Analog<'a, N> {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			let mut buf = [0; N];

			self.input.sample(&mut buf).await;

			info!("ADC sample: {}", buf);

			if buf == self.last_state {
				return;
			}
			self.last_state = buf;

			// TODO: Produce different events based on the number of inputs
			self.channel.publish(ReactorEvent::Joystick { x: buf[0], y: buf[1] }).await;
		})
	}
}
