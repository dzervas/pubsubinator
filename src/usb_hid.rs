use core::pin::Pin;

use alloc::boxed::Box;
use defmt::info;
use embassy_usb::Builder;
use embassy_usb::control::OutResponse;
use embassy_usb::class::hid::{HidWriter, ReportId, RequestHandler, State};
use futures::Future;
use static_cell::make_static;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::descriptor::SerializedDescriptor;

use reactor::RSubscriber;
use reactor::reactor_event::*;
use crate::nrf::UsbDriver;

pub struct UsbHid {
	writer: Option<HidWriter<'static, UsbDriver, 8>>,
	report: KeyboardReport,
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

		Self {
			writer: Some(writer),
			report: KeyboardReport {
				modifier: 0,
				reserved: 0,
				leds: 0,
				keycodes: [0; 6],
			},
		}
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
							if key > KeyCode::LCtrl && key < KeyCode::RGui {
								self.report.modifier |= 1 << (key as u8 - KeyCode::LCtrl as u8);
							} else if !self.report.keycodes.contains(&(key as u8)) {
								if let Some(pos) = self.report.keycodes.iter().position(|&k| k == KeyCode::None as u8) {
									self.report.keycodes[pos] = key as u8;
								}
							}
						},
						KeyEvent::Released(key) => {
							info!("Released: {:?}", key);
							if key > KeyCode::LCtrl && key < KeyCode::RGui {
								self.report.modifier &= 0 << (key as u8 - KeyCode::LCtrl as u8);
							} else if let Some(pos) = self.report.keycodes.iter().position(|&k| k == key as u8) {
								self.report.keycodes[pos] = 0;
							}
						},
						// _ => {
						// 	info!("Unhandled event: {:?}", value);
						// },
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
					return;
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
