#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
// #![feature(generic_const_exprs)]
#![feature(async_fn_in_trait)]
#![feature(trait_alias)]
#![feature(generic_arg_infer)]

extern crate alloc;
extern crate defmt_rtt;
extern crate panic_probe;

use core::mem;
use core::mem::size_of;

use alloc::vec;
use defmt::*;
use embassy_nrf::gpio::{Input, Pin, Pull, Output, Level, AnyPin};

use embassy_executor::{Spawner, task};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_time::Duration;
use embassy_time::Ticker;
use matrix::MATRIX_PERIOD;
use nrf_softdevice::ble::peripheral;
use nrf_softdevice::raw;
use nrf_softdevice::Softdevice;
use reactor::Consumer;
use reactor::Polled;
use reactor_event::{KeyCode, ReactorEvent};

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

use crate::matrix::Matrix;
use crate::reactor::Producer;
use crate::usb_hid::MyRequestHandler;

bind_interrupts!(struct Irqs {
	USBD => usb::InterruptHandler<peripherals::USBD>;
	POWER_CLOCK => usb::vbus_detect::InterruptHandler;
});

pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

pub const PUBSUB_CAPACITY: usize = 20 * size_of::<ReactorEvent>();
pub const PUBSUB_SUBSCRIBERS: usize = 4;
pub const PUBSUB_PUBLISHERS: usize = 4;
pub static CHANNEL: PubSubChannel<CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS> = PubSubChannel::new();

#[task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) {
	info!("USB task started");
	device.run().await;
	info!("USB task finished");
}

#[task]
async fn softdevice_task(sd: &'static Softdevice) {
	info!("SoftDevice task started");
	sd.run().await;
	info!("SoftDevice task finished");
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	info!("Hi from RustRover!");
	{
		use core::mem::MaybeUninit;
		const HEAP_SIZE: usize = 1024;
		static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
		unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
	}

	let p = embassy_nrf::init(Default::default());
	info!("Before heap init");

	let matrix: &'static mut Matrix<'static, Input<'static, AnyPin>, Output<'static, AnyPin>> = make_static!(matrix::Matrix {
		inputs: vec![
			Input::new(p.P0_04.degrade(), Pull::Down),
			Input::new(p.P0_30.degrade(), Pull::Down),
			Input::new(p.P1_14.degrade(), Pull::Down),
		],

		outputs: vec![
			Output::new(p.P0_03.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_28.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_29.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
		],

		keymap: vec![
			vec![KeyCode::Intl1, KeyCode::Intl2, KeyCode::Intl3],
			vec![KeyCode::Intl4, KeyCode::Intl5, KeyCode::Intl6],
			vec![KeyCode::Intl7, KeyCode::Intl8, KeyCode::Intl9],
		],
		last_state: vec![],
		direction: matrix::MatrixDirection::Row2Col,
		channel: CHANNEL.publisher().unwrap(),
	});
	matrix.setup().await;
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

	let state = make_static!(State::new());
	let writer = HidWriter::<_, 8>::new(&mut builder, state, hid_config);

	let usb = builder.build();

	spawner.spawn(usb_task(usb)).unwrap();

	let usb_hid = make_static!(usb_hid::UsbHid {
		writer: Some(writer),
		report: KeyboardReport {
			modifier: 0,
			reserved: 0,
			leds: 0,
			keycodes: [0; 6],
		},
		channel: CHANNEL.subscriber().unwrap(),
	});

	info!("USB HID consumer initialized");

	spawner.spawn(poller(matrix)).unwrap();
	spawner.spawn(subscriber(usb_hid)).unwrap();

	info!("Starting SoftDevice BLE shit");

	let sd_config = nrf_softdevice::Config {
		clock: Some(raw::nrf_clock_lf_cfg_t {
			source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
			rc_ctiv: 0,
			rc_temp_ctiv: 0,
			accuracy: raw::NRF_CLOCK_LF_ACCURACY_20_PPM as u8,
			// rc_ctiv: 16,
			// rc_temp_ctiv: 2,
			// accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
		}),
		conn_gap: Some(raw::ble_gap_conn_cfg_t {
			conn_count: 6,
			event_length: 24,
		}),
		conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
		gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t { attr_tab_size: 32768 }),
		gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
			adv_set_count: 1,
			periph_role_count: 3,
			central_role_count: 3,
			central_sec_count: 0,
			_bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
		}),
		gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
			p_value: b"HelloRust" as *const u8 as _,
			current_len: 9,
			max_len: 9,
			write_perm: unsafe { mem::zeroed() },
			_bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
		}),
		..Default::default()
	};

	let sd = Softdevice::enable(&sd_config);

	spawner.spawn(softdevice_task(sd)).unwrap();

	info!("Starting advertisement");

	let adv_data = &[
		0x02,
		raw::BLE_GAP_AD_TYPE_FLAGS as u8,
		raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
		0x03,
		raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE as u8,
		0x09,
		0x18,
		0x0a,
		raw::BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8,
		b'H',
		b'e',
		b'l',
		b'l',
		b'o',
		b'R',
		b'u',
		b's',
		b't',
	];
	#[rustfmt::skip]
	let scan_data = &[
		0x03, raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE as u8, 0x09, 0x18,
	];

	let mut config = peripheral::Config::default();
	config.interval = 50;
	let adv = peripheral::NonconnectableAdvertisement::ScannableUndirected {
		adv_data,
		scan_data,
	};
	unwrap!(peripheral::advertise(sd, adv, &config).await);

}

#[task]
async fn poller(poller: &'static mut dyn Polled) {
	let mut ticker = Ticker::every(Duration::from_millis(MATRIX_PERIOD));
	info!("Poller task started");

	loop {
		poller.poll().await;

		// Timer::after(Duration::from_millis(MATRIX_PERIOD)).await;
		ticker.next().await;
	}
}

#[task]
async fn subscriber(subscriber: &'static mut dyn Consumer) {
	info!("Subscriber task started");
	let mut listener = CHANNEL.subscriber().unwrap();
	loop {
		let msg = listener.next_message_pure().await;

		info!("[subscriber] Got a message: {:?}", msg);

		subscriber.push(msg).await;
	}
}
