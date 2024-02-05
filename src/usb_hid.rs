use core::ops::Deref;
use core::pin::Pin;

use alloc::boxed::Box;
use defmt::*;
use embassy_nrf::usb::vbus_detect::VbusDetect;
use embassy_usb::class::hid::{HidWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::Builder;
use futures::Future;
use static_cell::make_static;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use crate::nrf::UsbDriver;
use crate::VBUS_DETECT;
use reactor::reactor_event::*;
use reactor::RSubscriber;

pub struct UsbHid {
	writer: Option<HidWriter<'static, UsbDriver, 8>>,
}

impl UsbHid {
	pub fn new(builder: &mut Builder<'static, UsbDriver>) -> Self {
		info!("Initializing USB HID");

		let request_handler = make_static!(MyRequestHandler {});

		// Create classes on the builder.
		let hid_config = embassy_usb::class::hid::Config {
			report_descriptor: KeyboardReport::desc(),
			request_handler: Some(request_handler),
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
				_ => {},
			}
		})
	}
}

pub struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
	fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
		info!("Get report for {:?}", id);
		None
	}

	fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
		info!("Set report for {:?}: {=[u8]}", id, data);
		OutResponse::Accepted
	}

	fn set_idle_ms(&self, id: Option<ReportId>, dur: u32) {
		info!("Set idle rate for {:?} to {:?}", id, dur);
	}

	fn get_idle_ms(&self, id: Option<ReportId>) -> Option<u32> {
		info!("Get idle rate for {:?}", id);
		None
	}
}
