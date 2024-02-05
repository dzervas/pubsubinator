use core::pin::Pin;

use defmt::*;
use alloc::boxed::Box;
use futures::prelude::Future;
use reactor::{KeyCode, KeyEvent, KeyModifiers, ReactorEvent};
use reactor::middleware::Middleware;
use usbd_hid::descriptor::KeyboardReport;

#[derive(Debug, Default)]
pub struct KeyboardReportMid {
	modifiers: KeyModifiers,
	keys: [KeyCode; 6],
}

impl KeyboardReportMid {
	pub fn into_event(&self) -> ReactorEvent {
		ReactorEvent::KeyboardReport {
			modifier: self.modifiers,
			keycodes: self.keys
		}
	}
}

impl Middleware for KeyboardReportMid {
	fn process(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = Option<ReactorEvent>> + '_>> {
		Box::pin(async move {
			match value {
				ReactorEvent::Key(code) => {
					match code {
						KeyEvent::Pressed(key) => {
							if key > KeyCode::LCtrl && key < KeyCode::RGui {
								let modifiers = <KeyModifiers as Into<u8>>::into(self.modifiers) | (1 << (key as u8 - KeyCode::LCtrl as u8));
								self.modifiers = KeyModifiers::from(modifiers);
							} else if !self.keys.contains(&key) {
								if let Some(pos) = self.keys.iter().position(|&k| k == KeyCode::None) {
									self.keys[pos] = key;
								}
							}

							Some(self.into_event())
						},
						KeyEvent::Released(key) => {
							info!("Released: {:?}", key);
							if key > KeyCode::LCtrl && key < KeyCode::RGui {
								self.modifiers = (<KeyModifiers as Into<u8>>::into(self.modifiers) & (0 << (key as u8 - KeyCode::LCtrl as u8))).into();
							} else if let Some(pos) = self.keys.iter().position(|&k| k == key) {
								self.keys[pos] = KeyCode::None;
							}

							Some(self.into_event())
						},
					}
				},
				_ => None
			}
		})
	}
}

impl Into<KeyboardReport> for KeyboardReportMid {
	fn into(self) -> KeyboardReport {
		// TODO: Make this a generic
		let keycodes = [self.keys[0] as u8, self.keys[1] as u8, self.keys[2] as u8, self.keys[3] as u8, self.keys[4] as u8, self.keys[5] as u8];

		KeyboardReport {
			modifier: self.modifiers.into(),
			reserved: 0,
			leds: 0,
			keycodes,
		}
	}
}
