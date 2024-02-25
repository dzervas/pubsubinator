#![no_std]
use serde::ser::{Serialize, SerializeTuple, Serializer};
use usbd_hid_macros::gen_hid_descriptor;
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

pub mod constants;
pub mod structs;

#[macro_export]
macro_rules! hid {
	($($name:ident = collection($collection_type:ident, $usage_page:ident, $usage:ident) { $($inner:tt)* }),*) => {
		pub struct HIDReportMap {
			// pub collections: &'static [Collection],
		}

		// $(
			// pub struct Collection$name {
			// 	pub collection_type: constants::CollectionID,
			// 	pub usage_page: constants::UsagePageID,
			// 	pub usages: &'static [Usage],
			// }
		// )*
	};
	($name:ident = usage($($key:ident = $value:ident),*)) => {
		// Usage {
		// 	$($key: constants::UsageID::$value),*
		// 	..Default::default()
		// }
	};
}

hid! {
	keyboard = collection(Application, GenericDesktop, Keyboard) {
		modifier = usage(usage_page = Keyboard, usage_min = 0xE0, usage_max = 0xE7),
		reserved = usage(),
		leds = usage(usage_page = LED, usage_min = 0x01, usage_max = 0x05),
		keyboard = usage(usage_page = Keyboard, usage_min = 0x00, usage_max = 0xDD)
	}
}

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
        };
        (usage_min = 0x00, usage_max = 0xFF) = {
            #[item_settings constant,variable,absolute] reserved=input;
        };
        (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
            #[packed_bits 5] #[item_settings data,variable,absolute] leds=output;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
            #[item_settings data,array,absolute] keycodes=input;
        };
    }
)]
#[allow(dead_code)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
}
