use embassy_executor::task;
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_nrf::peripherals;
use embassy_nrf::usb::Driver;
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::pac::Interrupt;
use embassy_usb::{Config, Builder};
use static_cell::make_static;
use defmt::*;

use crate::{Irqs, VBUS_DETECT};

// TODO: Choose between SoftwareVbusDetect and HardwareVbusDetect - hardware if SoftDevice is not used
pub type UsbDriver = Driver<'static, peripherals::USBD, &'static SoftwareVbusDetect>;

#[task]
pub async fn usb_task(builder: Builder<'static, UsbDriver>) {
	let mut device = builder.build();

	info!("USB task started");
	device.run().await;
	info!("USB task finished");
}

pub fn usb_init(p_usbd: peripherals::USBD) -> Builder<'static, UsbDriver> {
	// Required to work with the SoftDevice
	Interrupt::USBD.set_priority(Priority::P2);

	let driver = Driver::new(p_usbd, Irqs, &*VBUS_DETECT);

	let mut usb_config = Config::new(0xc0de, 0xcafe);
	usb_config.manufacturer = Some("DZervas");
	usb_config.product = Some("PubSubinator");
	usb_config.serial_number = Some(env!("CARGO_PKG_VERSION"));
	usb_config.max_power = 100;
	usb_config.max_packet_size_0 = 64;
	usb_config.supports_remote_wakeup = true;

	let builder = Builder::new(
		driver,
		usb_config,
		&mut make_static!([0; 256])[..],
		&mut make_static!([0; 256])[..],
		&mut make_static!([0; 256])[..],
		&mut make_static!([0; 128])[..],
		&mut make_static!([0; 128])[..],
	);

	builder
}
