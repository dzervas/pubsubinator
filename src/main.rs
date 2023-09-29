#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
// #![feature(generic_const_exprs)]
#![feature(async_fn_in_trait)]
#![feature(trait_alias)]

extern crate alloc;
extern crate defmt_rtt;
extern crate panic_probe;

use alloc::boxed::Box;
use alloc::vec;
use defmt::*;
use embassy_nrf::gpio::{Input, Pin, Pull, Output, Level, AnyPin};

use embassy_executor::{Spawner, task};

use reactor_event::KeyCode;

use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub mod matrix;
pub mod reactor;
pub mod reactor_event;
pub mod usb_hid;

use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::{bind_interrupts, peripherals, usb};
use embassy_usb::class::hid::{HidWriter, State};
use embassy_usb::{Builder, Config, UsbDevice};
use static_cell::make_static;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::descriptor::SerializedDescriptor;

use crate::reactor::{react, Producer};
use crate::usb_hid::MyRequestHandler;

bind_interrupts!(struct Irqs {
	USBD => usb::InterruptHandler<peripherals::USBD>;
	POWER_CLOCK => usb::vbus_detect::InterruptHandler;
});

pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

#[task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) {
	info!("USB task started");
	device.run().await;
	info!("USB task finished");
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	info!("Hi!");
	{
		use core::mem::MaybeUninit;
		const HEAP_SIZE: usize = 1024;
		static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
		unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
	}

	let p = embassy_nrf::init(Default::default());
	info!("Before heap init");

	let mut matrix: matrix::Matrix<Input<'static, AnyPin>, Output<'static, AnyPin>> = matrix::Matrix {
		inputs: vec![
		// 	Input::new(p.P0_03.degrade(), Pull::Down),
		// 	Input::new(p.P0_30.degrade(), Pull::Down),
		// 	Input::new(p.P1_14.degrade(), Pull::Down),
			Input::new(p.P0_04.degrade(), Pull::Down),
			Input::new(p.P0_28.degrade(), Pull::Down),
			Input::new(p.P0_29.degrade(), Pull::Down),
		],

		outputs: vec![
		// 	Output::new(p.P0_04.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
		// 	Output::new(p.P0_28.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
		// 	Output::new(p.P0_29.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_03.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_30.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P1_14.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
		],

		keymap: vec![
			KeyCode::Intl1,
			KeyCode::Intl2,
			KeyCode::Intl3,
			KeyCode::Intl4,
			KeyCode::Intl5,
			KeyCode::Intl6,
			KeyCode::Intl7,
			KeyCode::Intl8,
			KeyCode::Intl9,
		],
		last_state: vec![],
		event_buffer: vec![],
		direction: matrix::MatrixDirection::Col2Row
	};
	matrix.setup();
	info!("Matrix initialized");

	// -- Setup USB HID consumer --

	let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

	let mut usb_config = Config::new(0xc0de, 0xcafe);
	usb_config.manufacturer = Some("DZervas");
	usb_config.product = Some("RustRover");
	usb_config.serial_number = Some(env!("CARGO_PKG_VERSION"));
	usb_config.max_power = 100;
	usb_config.max_packet_size_0 = 64;
	usb_config.supports_remote_wakeup = true;

	let mut builder = Builder::new(
		driver,
		usb_config,
		&mut make_static!([0; 256])[..],
		&mut make_static!([0; 256])[..],
		&mut make_static!([0; 256])[..],
		&mut make_static!([0; 128])[..],
		&mut make_static!([0; 128])[..],
	);

	info!("USB HID consumer initialized");

	let request_handler = make_static!(MyRequestHandler {});

	// Create classes on the builder.
	let hid_config = embassy_usb::class::hid::Config {
		report_descriptor: KeyboardReport::desc(),
		request_handler: Some(request_handler),
		poll_ms: 60,
		max_packet_size: 64,
	};

	// let mut state = State::new();
	let state = make_static!(State::new());
	let writer = HidWriter::<_, 8>::new(&mut builder, state, hid_config);

	let usb = builder.build();

	spawner.spawn(usb_task(usb)).unwrap();

	let usb_hid = usb_hid::UsbHid {
		writer: Some(writer),
		report: KeyboardReport {
			modifier: 0,
			reserved: 0,
			leds: 0,
			keycodes: [0; 6],
		},
	};

	info!("USB HID consumer initialized");

	// let remote_wakeup: Signal<CriticalSectionRawMutex, bool> = Signal::new();

	let reactor = make_static!(reactor::Reactor {
		producers: vec![Box::new(matrix)],
		consumers: vec![Box::new(usb_hid)],
	});

	info!("Reactor initialized");

	// loop {
		// info!("Reacting");
		// reactor.react().await;
	// }

	spawner.spawn(react(reactor)).unwrap();
}
