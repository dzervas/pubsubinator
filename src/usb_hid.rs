use core::pin::Pin;

use alloc::boxed::Box;
use defmt::info;
use embassy_usb::control::OutResponse;
use embassy_usb::class::hid::{HidWriter, ReportId, RequestHandler};
use futures::Future;
use usbd_hid::descriptor::KeyboardReport;

use crate::reactor::Consumer;
use crate::reactor_event::*;
use crate::UsbDriver;

pub struct UsbHid {
	pub writer: Option<HidWriter<'static, UsbDriver, 8>>,
	pub report: KeyboardReport,
}

impl Consumer for UsbHid {
	fn setup() -> Self
	where
		Self: Sized,
	{
		Self {
			writer: None,
			report: KeyboardReport {
				modifier: 0,
				reserved: 0,
				leds: 0,
				keycodes: [0; 6],
			},
		}
	}

	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		if self.writer.is_none() {
			info!("USB HID writer is not ready");
			return Box::pin(async {
				()
			});
		}

		Box::pin(async move {
			match value {
				ReactorEvent::Key(code) => {
					match code {
						KeyEvent::Pressed(key) => {
							info!("Pressed: {:?}", key);
							self.report.keycodes[0] = key as u8;
						},
						KeyEvent::Released(key) => {
							info!("Released: {:?}", key);
							self.report.keycodes[0] = 0;
						},
						_ => {
							info!("Unhandled event: {:?}", value);
						},
					}
				},
				// ReactorEvent::Locks { caps, num, scroll } => {
				// 	self.report.modifier = 0;
				// 	self.report.keycodes[0] = caps as u8;
				// },
				// ReactorEvent::Mouse { x, y } => {
				// 	info!("Unhandled event: {:?}", value);
				// },
				_ => {
					info!("Unhandled event: {:?}", value);
				},
			}

			self.writer.as_mut().unwrap().ready().await;

			self.writer.as_mut().unwrap().write_serialize(&self.report).await.unwrap();
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
