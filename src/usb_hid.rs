use core::ops::Deref;
use core::pin::Pin;

use alloc::boxed::Box;
use defmt::*;
use embassy_nrf::usb::vbus_detect::VbusDetect;
use embassy_usb::class::hid::{HidWriter, State};
use embassy_usb::Builder;
use futures::Future;
use static_cell::make_static;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use crate::nrf::UsbDriver;
use crate::report_maps::SpaceMouseReport;
use crate::VBUS_DETECT;
use reactor::reactor_event::*;
use reactor::RSubscriber;

pub struct UsbHid {
	writer: Option<HidWriter<'static, UsbDriver, 8>>,
}

impl UsbHid {
	pub fn new<D: SerializedDescriptor>(builder: &mut Builder<'static, UsbDriver>) -> Self {
		info!("Initializing USB HID");

		// Create classes on the builder.
		let hid_config = embassy_usb::class::hid::Config {
			report_descriptor: D::desc(),
			request_handler: None,
			poll_ms: 60,
			max_packet_size: 64,
		};

		let state = make_static!(State::new());
		let writer = HidWriter::<_, 8>::new(builder, state, hid_config);

		Self { writer: Some(writer) }
	}
}

impl RSubscriber for UsbHid {
	fn is_supported(&self, event: ReactorEvent) -> bool {
		(match event {
			ReactorEvent::Key(_) => true,
			// ReactorEvent::Locks { caps, num, scroll } => true,
			// ReactorEvent::Mouse { x, y } => true,
			_ => false,
		}) && self.writer.is_some()
			&& VBUS_DETECT.deref().is_usb_detected()
	}

	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async move {
			match value {
				ReactorEvent::KeyboardReport { modifier, keycodes } => {
					let report = KeyboardReport {
						modifier: modifier.into(),
						reserved: 0,
						leds: 0,
						// TODO: Make this a generic
						keycodes: [
							keycodes[0].into(),
							keycodes[1].into(),
							keycodes[2].into(),
							keycodes[3].into(),
							keycodes[4].into(),
							keycodes[5].into(),
						],
					};

					// self.writer.as_mut().unwrap().ready().await;
					match self.writer.as_mut().unwrap().write_serialize(&report).await {
						Ok(_) => {},
						Err(e) => warn!("Error writing to USB HID: {:?}", e),
					}
				},
				ReactorEvent::Joystick6DoF { x, y, z, rx, ry, rz } => {
					let report = SpaceMouseReport {
						x,
						y,
						z,
						rx,
						ry,
						rz,
						buttons: 0,
					};

					match self.writer.as_mut().unwrap().write_serialize(&report).await {
						Ok(_) => {},
						Err(e) => warn!("Error writing to USB HID: {:?}", e),
					}
				}
				_ => return,
			}
		})
	}
}
