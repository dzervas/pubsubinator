#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
// #![feature(generic_const_exprs)]
#![feature(async_fn_in_trait)]

extern crate alloc;
extern crate defmt_rtt;
extern crate panic_probe;

use alloc::boxed::Box;
use alloc::vec;
use embassy_futures::select::{select, Either};
use embassy_nrf::gpio::{Input, Pin, Pull, Output, Level, AnyPin};
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::{bind_interrupts, peripherals, usb};

use embassy_executor::{Spawner, task};

use defmt::*;
use embassy_nrf::usb::Driver;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_usb::{Config, Builder};
use embassy_usb::class::hid::{State, HidReaderWriter};
use embedded_hal::digital::v2::InputPin;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::descriptor::SerializedDescriptor;

use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub mod keyboard;
pub mod matrix;
pub mod reactor;
pub mod reactor_event;

bind_interrupts!(struct Irqs {
	USBD => usb::InterruptHandler<peripherals::USBD>;
	POWER_CLOCK => usb::vbus_detect::InterruptHandler;
});

#[task]
async fn main_task() {
	let p = embassy_nrf::init(Default::default());

	let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

	let mut config = Config::new(0xc0de, 0xcafe);
	config.manufacturer = Some("Embassy");
	config.product = Some("HID keyboard example");
	config.serial_number = Some("12345678");
	config.max_power = 100;
	config.max_packet_size_0 = 64;
	config.supports_remote_wakeup = true;

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
		config,
		&mut device_descriptor,
		&mut config_descriptor,
		&mut bos_descriptor,
		&mut msos_descriptor,
		&mut control_buf,
	);

	// builder.handler(&mut device_handler);

	// Create classes on the builder.
	let config = embassy_usb::class::hid::Config {
		report_descriptor: KeyboardReport::desc(),
		// request_handler: Some(&request_handler),
		request_handler: None,
		poll_ms: 60,
		max_packet_size: 64,
	};
	let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);

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

	let mut button = Input::new(p.P0_11.degrade(), Pull::Up);

	let matrix: matrix::Matrix<Input<'static, AnyPin>, Output<'static, AnyPin>> = matrix::Matrix {
		inputs: vec![
			Input::new(p.P0_03.degrade(), Pull::Down),
			Input::new(p.P0_28.degrade(), Pull::Down),
			Input::new(p.P0_29.degrade(), Pull::Down),
		],

		outputs: vec![
			Output::new(p.P0_04.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_30.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_14.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
		],

		keymap: vec![
			keyboard::KeyCode::INT1,
			keyboard::KeyCode::INT2,
			keyboard::KeyCode::INT3,
			keyboard::KeyCode::INT4,
			keyboard::KeyCode::INT5,
			keyboard::KeyCode::INT6,
			keyboard::KeyCode::INT7,
			keyboard::KeyCode::INT8,
			keyboard::KeyCode::INT9,
		],
		last_state: vec![],
		direction: matrix::MatrixDirection::Col2Row
	};

	let reactor = reactor::Reactor {
		producers: vec![],
		consumers: vec![],
	};

	// TODO: Setup USB HID consumer
	// TODO: Setup matrix producer

	loop {
		reactor.react().await;
	}
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	spawner.spawn(main_task()).unwrap();
}
