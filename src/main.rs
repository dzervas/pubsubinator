#![no_std]
#![no_main]
#![feature(async_fn_in_trait)]
#![feature(generic_arg_infer)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

extern crate alloc;
extern crate defmt_rtt;
extern crate panic_probe;
extern crate embassy_nrf;

use core::mem;
use core::mem::size_of;

use defmt::*;
use embassy_nrf::gpio::{Input, Pin, Pull, Output, Level, AnyPin};

use embassy_executor::{task, Spawner};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_time::Duration;
use embassy_time::Ticker;
use lazy_static::lazy_static;
use matrix::MATRIX_PERIOD;
use reactor::RSubscriber;
use reactor::Polled;
use reactor::reactor_event::{KeyCode, ReactorEvent};
use reactor::reactor_event::KeyCodeInt::Key;

use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

use embassy_nrf::interrupt::Priority;
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use nrf_softdevice::raw;
use nrf_softdevice::SocEvent;
use nrf_softdevice::Softdevice;
use embassy_nrf::{bind_interrupts, peripherals, usb, saadc};
use static_cell::make_static;

pub mod matrix;
pub mod analog_nrf;
pub mod usb_hid;
pub mod ble_hid;
pub mod nrf;
pub mod keymap_mid;

#[allow(unused_imports)]
use crate::analog_nrf::Analog;
use crate::keymap_mid::KEYMAP_PERIOD;
use crate::matrix::Matrix;
use crate::nrf::usb_init;
use crate::nrf::usb_task;
use crate::ble_hid::BleHid;
use crate::usb_hid::UsbHid;

bind_interrupts!(struct Irqs {
	USBD => usb::InterruptHandler<peripherals::USBD>;
	SAADC => saadc::InterruptHandler;
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
	info!("PubSubinator v{}", env!("CARGO_PKG_VERSION"));
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

	let matrix: &'static mut Matrix<'static, Input<'static, AnyPin>, Output<'static, AnyPin>, _, _> = make_static!(matrix::Matrix::new(
		[
			Input::new(p.P0_04.degrade(), Pull::Down),
			Input::new(p.P0_30.degrade(), Pull::Down),
			Input::new(p.P1_14.degrade(), Pull::Down),
		],
		[
			Output::new(p.P0_03.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_28.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
			Output::new(p.P0_29.degrade(), Level::Low, embassy_nrf::gpio::OutputDrive::Standard),
		],
		matrix::MatrixDirection::Row2Col,
	));
	spawner.spawn(poller_task(matrix)).unwrap();
	info!("Matrix publisher initialized");

	// -- Setup Keymap middleware --
	let keymap = make_static!(keymap_mid::Keymap::new(
		[[
			[Key(KeyCode::Intl1), Key(KeyCode::Intl2), Key(KeyCode::Intl3)],
			[Key(KeyCode::Intl4), Key(KeyCode::Intl5), Key(KeyCode::Intl6)],
			[Key(KeyCode::Intl7), Key(KeyCode::Intl8), Key(KeyCode::Intl9)],
		]],
		2000 / KEYMAP_PERIOD as u16,
	));
	info!("Keymap middleware initialized");

	// -- Setup Analog publisher --

	// let analog = make_static!(Analog::new(p.SAADC, [
	// 	Into::<saadc::AnyInput>::into(p.P0_03),
	// 	Into::<saadc::AnyInput>::into(p.P0_04),
	// ]));
	// spawner.spawn(poller_task(analog)).unwrap();

	// -- Setup USB HID consumer --
	let mut usb_builder = usb_init(p.USBD);
	let usb_hid = make_static!(UsbHid::new(&mut usb_builder));

	spawner.spawn(usb_task(usb_builder)).unwrap();
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

	let subs_task = reactor_macros::subscribers_task!(CHANNEL, [usb_hid, keymap]);
	spawner.spawn(subs_task).unwrap();
}

#[task]
async fn poller_task(poller: &'static mut dyn Polled) {
	let mut ticker = Ticker::every(Duration::from_millis(MATRIX_PERIOD));
	info!("Poller task started");

	loop {
		// TODO: Turn this into a join of all pollers
		poller.poll().await;
		ticker.next().await;
		// Timer::after(Duration::from_millis(100)).await;
	}
}

#[task]
async fn subscriber_task(mut subscribers: [&'static mut dyn RSubscriber; 2]) {
	info!("Subscriber task started");
	let mut listener = CHANNEL.subscriber().unwrap();
	loop {
		let msg = listener.next_message_pure().await;

		info!("[subscriber] Got a message: {:?}", msg);

		// let valid_subs: Vec<_> = subscribers.iter().filter(|s| s.is_supported(msg.clone())).collect();

		// join!(valid_subs.map(|s| s.push(msg.clone())));
		for sub in &mut subscribers {
			if sub.is_supported(msg.clone()) {
				sub.push(msg.clone()).await;
			}
		}
	}
}
