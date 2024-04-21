use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use defmt::*;

use embassy_nrf::saadc::{Gain, Reference, Resistor, Saadc, Time};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;

// TODO: Use a generics instead of nrf-specifics
use embassy_nrf::peripherals::SAADC;
use embassy_nrf::saadc::Input;
use embassy_nrf::Peripheral;

use crate::{Irqs, PUBSUB_CAPACITY, PUBSUB_PUBLISHERS, PUBSUB_SUBSCRIBERS};
use reactor::reactor_event::*;
use reactor::{Polled, RPublisher};

pub struct Analog<'a, const N: usize> {
	input: Saadc<'a, N>,
	last_state: [i16; N],
	channel:
		Publisher<'a, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>,
}

impl<'a, const N: usize> Analog<'a, N> {
	pub fn new(p_saadc: SAADC, input: [impl Peripheral<P = impl Input>; N]) -> Self {
		let config = embassy_nrf::saadc::Config::default();
		let mut input_iter = input.into_iter();
		let channel_config = [(); N].map(|_| {
			let mut cc = embassy_nrf::saadc::ChannelConfig::single_ended(input_iter.next().unwrap());
			cc.gain = Gain::GAIN1;
			cc.reference = Reference::VDD1_4;
			cc.resistor = Resistor::VDD1_2;
			cc.time = Time::_3US;
			cc
		});
		let saadc: Saadc<'a, N> = Saadc::new(p_saadc, Irqs, config, channel_config);

		Self {
			input: saadc,
			last_state: [0; N],
			channel: crate::CHANNEL.publisher().unwrap(),
		}
	}

	async fn _poll_internal(&mut self) -> Option<[i16; N]> {
		let mut buf = [0; N];

		// TODO: It's VERY slow - about 1s
		self.input.sample(&mut buf).await;

		info!("ADC sample: {}", buf);

		if buf == self.last_state {
			return None;
		}
		self.last_state = buf;

		// TODO: Maybe produce different events based on the number of inputs
		Some(buf)
	}
}

impl<'a, const N: usize> RPublisher for Analog<'a, N> {}

impl<'a> Polled for Analog<'a, 1> {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			if let Some(buf) = self._poll_internal().await {
				self.channel.publish(ReactorEvent::Potentiometer { v: buf[0] }).await;
			}
		})
	}
}

impl<'a> Polled for Analog<'a, 2> {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			if let Some(buf) = self._poll_internal().await {
				self.channel
					.publish(ReactorEvent::Joystick { x: buf[0], y: buf[1] })
					.await;
			}
		})
	}
}

impl<'a> Polled for Analog<'a, 3> {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			if let Some(buf) = self._poll_internal().await {
				self.channel
					.publish(ReactorEvent::FullJoystick {
						x: buf[0],
						y: buf[1],
						z: buf[2],
					})
					.await;
			}
		})
	}
}

impl<'a> Polled for Analog<'a, 6> {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async {
			if let Some(buf) = self._poll_internal().await {
				self.channel
					.publish(ReactorEvent::Joystick6DoF {
						x: buf[0],
						y: buf[1],
						z: buf[2],
						rx: buf[3],
						ry: buf[4],
						rz: buf[5],
					})
					.await;
			}
		})
	}
}
