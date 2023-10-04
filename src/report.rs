use usbd_hid::descriptor::{KeyboardReport, AsInputReport};

pub struct Report {
	pub modifier: u8,
	pub reserved: u8,
	pub leds: u8,
	pub keycodes: [u8; 6],
}

impl Into<KeyboardReport> for Report {
	fn into(self) -> KeyboardReport {
		KeyboardReport {
			modifier: self.modifier,
			reserved: self.reserved,
			leds: self.leds,
			keycodes: self.keycodes,
		}
	}
}

impl From<KeyboardReport> for Report {
	fn from(report: KeyboardReport) -> Self {
		Self {
			modifier: report.modifier,
			reserved: report.reserved,
			leds: report.leds,
			keycodes: report.keycodes,
		}
	}
}

impl AsInputReport for Report {
	const ID: u8 = 1;
}
