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
extern crate embassy_nrf;

use core::mem;
use core::mem::size_of;

use alloc::vec;
use defmt::*;
use embassy_nrf::gpio::{Input, Pin, Pull, Output, Level, AnyPin};

use embassy_executor::{task, Spawner};
use embassy_nrf::interrupt::Priority;
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_time::Duration;
use embassy_time::Ticker;
use lazy_static::lazy_static;
use matrix::MATRIX_PERIOD;
use nrf_softdevice::raw;
use nrf_softdevice::SocEvent;
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
pub mod ble_hid;
pub mod nrf;
// pub mod report;

use embassy_nrf::{bind_interrupts, peripherals, usb};
use static_cell::make_static;
// use usbd_hid::descriptor::KeyboardReport;

use crate::matrix::Matrix;
use crate::nrf::usb_init;
use crate::nrf::usb_task;
use crate::reactor::Producer;
use crate::ble_hid::BleHid;
use crate::usb_hid::UsbHid;

bind_interrupts!(struct Irqs {
	USBD => usb::InterruptHandler<peripherals::USBD>;
});

pub const PUBSUB_CAPACITY: usize = 20 * size_of::<ReactorEvent>();
pub const PUBSUB_SUBSCRIBERS: usize = 4;
pub const PUBSUB_PUBLISHERS: usize = 4;
pub static CHANNEL: PubSubChannel<CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS> = PubSubChannel::new();
lazy_static! {
	// TODO: Add support for HardwareVbusDetect as well to avoid needing the SoftDevice
	pub static ref VBUS_DETECT: SoftwareVbusDetect = SoftwareVbusDetect::new(true, true);
}

#[task]
async fn softdevice_task(sd: &'static Softdevice) {
	info!("SoftDevice task started");
	sd.run_with_callback(|event: SocEvent| {
		info!("SoftDevice event: {:?}", event);

		match event {
			SocEvent::PowerUsbRemoved => VBUS_DETECT.detected(false),
			SocEvent::PowerUsbDetected => VBUS_DETECT.detected(true),
			SocEvent::PowerUsbPowerReady => VBUS_DETECT.ready(),
			_ => {}
		};
	}).await;
	info!("SoftDevice task finished");
}

#[task]
async fn ble_hid_task(ble_hid: &'static mut BleHid<'static>) {
	info!("BLE HID task started");
	ble_hid.run().await;
	info!("BLE HID task finished");
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

	let mut config = embassy_nrf::config::Config::default();
	config.gpiote_interrupt_priority = Priority::P2;
	config.time_interrupt_priority = Priority::P2;
	let p = embassy_nrf::init(config);
	info!("Before heap init");

	// -- Setup Matrix publisher --

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
	spawner.spawn(poller(matrix)).unwrap();
	info!("Matrix publisher initialized");

	// -- Setup USB HID consumer --
	let mut usb_builder = usb_init(p.USBD);
	let usb_hid = make_static!(UsbHid::new(&mut usb_builder));

	spawner.spawn(usb_task(usb_builder)).unwrap();
	spawner.spawn(subscriber(usb_hid)).unwrap();
	info!("USB HID consumer initialized");

	// -- Setup SoftDevice --
	info!("Starting SoftDevice BLE shit");

	let sd_config = nrf_softdevice::Config {
		clock: Some(raw::nrf_clock_lf_cfg_t {
			// Use external crystal
			source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
			// Need to be 0 for external crystal
			rc_ctiv: 0,
			// Need to be 0 for external crystal
			rc_temp_ctiv: 0,
			// Crystal accuracy - why 20?
			accuracy: raw::NRF_CLOCK_LF_ACCURACY_20_PPM as u8,
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
	// let server = unwrap!(ble_hid::Server::new(sd));
	spawner.spawn(softdevice_task(sd)).unwrap();
	info!("SoftDevice initialized");

	// -- Setup BLE HID consumer --
	// let services = ServiceBuilder::new(sd, Uuid::new_16(0x1812)).unwrap()
	// 	.add_characteristic(Uuid::new_16(0x2A4B), ble_hid::HidReportAttribute::new(), Metadata::new(Properties {
	// 		read: true,
	// 		..Default::default()
	// 	})).unwrap()
	// 	.build();

	// let ble_hid = make_static!(BleHid {
	// 	softdevice: sd,
	// 	server,
	// 	report: KeyboardReport {
	// 		modifier: 0,
	// 		reserved: 0,
	// 		leds: 0,
	// 		keycodes: [0; 6],
	// 	},
	// 	channel: CHANNEL.subscriber().unwrap(),
	// });

	// info!("Starting advertisement");
	// spawner.spawn(ble_hid_task(ble_hid)).unwrap();
	// spawner.spawn(subscriber(ble_hid)).unwrap();
}

#[task]
async fn poller(poller: &'static mut dyn Polled) {
	let mut ticker = Ticker::every(Duration::from_millis(MATRIX_PERIOD));
	info!("Poller task started");

	loop {
		poller.poll().await;
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
