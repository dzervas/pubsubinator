pub use usbd_hid::descriptor::*;
use serde::ser::{Serialize, SerializeTuple, Serializer};

#[gen_hid_descriptor(
	(collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MULTI_AXIS_CONTROLLER) = {
		// TODO: Logical is -500 to 500, physical is -32768 to 32767
		(collection = PHYSICAL,) = {
			(usage = X,) = {
				#[item_settings variable,absolute] x=input;
			};
			(usage = Y,) = {
				#[item_settings variable,absolute] y=input;
			};
			(usage = Z,) = {
				#[item_settings variable,absolute] z=input;
			};
		};
		(collection = PHYSICAL,) = {
			(usage = 0x33,) = {
				#[item_settings variable,absolute] rx=input;
			};
			(usage = 0x34,) = {
				#[item_settings variable,absolute] ry=input;
			};
			(usage = 0x35,) = {
				#[item_settings variable,absolute] rz=input;
			};
		};
		(collection = PHYSICAL,) = {
			(usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_8) = { // max 24 buttons normally
				#[packed_bits 8] #[item_settings data,variable,absolute] buttons=input;
			};
		};
	}
)]
pub struct SpaceMouseReport {
	pub x: i16,
	pub y: i16,
	pub z: i16,
	pub rx: i16,
	pub ry: i16,
	pub rz: i16,
	pub buttons: u8,
}
