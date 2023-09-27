use core::pin::Pin;

use alloc::boxed::Box;
use defmt::unwrap;
use embassy_nrf::usb::Driver;
use embassy_futures::select::{select, Either};
use embassy_nrf::gpio::{Input, Pin as GPIOPin, Pull, Output, Level, AnyPin};
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::{bind_interrupts, peripherals, usb};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_usb::driver::Driver as UsbDriver;
use embassy_usb::{Config, Builder};
use embassy_usb::class::hid::{State, HidReaderWriter, HidReader, HidWriter};
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::descriptor::SerializedDescriptor;
use futures::Future;

use crate::reactor::Consumer;
use crate::reactor_event::*;

bind_interrupts!(struct Irqs {
	USBD => usb::InterruptHandler<peripherals::USBD>;
	POWER_CLOCK => usb::vbus_detect::InterruptHandler;
});

pub struct UsbHid<'a, T: UsbDriver<'a>> {
	// hid: HidReaderWriter<'a, T, 1, 8>,
	reader: HidReader<'a, T, 1>,
	writer: HidWriter<'a, T, 8>,
	remote_wakeup: Signal<CriticalSectionRawMutex, bool>,
}

impl<'a, T: UsbDriver<'a>> Consumer for UsbHid<'a, T> {
	fn setup() -> Self where Self: Sized {
		let p = embassy_nrf::init(Default::default());
		let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

		let mut usb_config = Config::new(0xc0de, 0xcafe);
		usb_config.manufacturer = Some("DZervas");
		usb_config.product = Some("RustRover");
		usb_config.serial_number = Some(env!("CARGO_PKG_VERSION"));
		usb_config.max_power = 100;
		usb_config.max_packet_size_0 = 64;
		usb_config.supports_remote_wakeup = true;

		let mut device_descriptor = [0; 256];
		let mut config_descriptor = [0; 256];
		let mut bos_descriptor = [0; 256];
		let mut msos_descriptor = [0; 256];
		let mut control_buf = [0; 64];
		// let request_handler = MyRequestHandler {};
		// let mut device_handler = MyDeviceHandler::new();

		let mut state = State::new();

		let mut builder = Builder::new(
			driver,
			usb_config,
			&mut device_descriptor,
			&mut config_descriptor,
			&mut bos_descriptor,
			&mut msos_descriptor,
			&mut control_buf,
		);

		// builder.handler(&mut device_handler);

		// Create classes on the builder.
		let hid_config = embassy_usb::class::hid::Config {
			report_descriptor: KeyboardReport::desc(),
			// request_handler: Some(&request_handler),
			request_handler: None,
			poll_ms: 60,
			max_packet_size: 64,
		};

		let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, hid_config);

		// Build the builder.
		let mut usb = builder.build();

		let remote_wakeup: Signal<CriticalSectionRawMutex, bool> = Signal::new();

		// Run the USB device.
		let usb_fut = async {
			loop {
				usb.run_until_suspend().await;
				match select(usb.wait_resume(), remote_wakeup.wait()).await {
					Either::First(_) => (),
					Either::Second(_) => unwrap!(usb.remote_wakeup().await),
				}
			}
		};

		let (reader, mut writer) = hid.split();

		// self.reader = reader;
		// self.writer = writer;

		Self {
			reader,
			writer,
			remote_wakeup,
		}
	}

	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()>>> {
		Box::pin(async {
			()
		})
	}
}
