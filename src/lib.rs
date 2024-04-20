#![no_std]
// make_static! macro requires this
#![feature(type_alias_impl_trait)]

extern crate alloc;
#[cfg(feature = "debug")]
extern crate defmt_rtt;
extern crate embassy_nrf;
extern crate panic_probe;

use core::mem::{self, size_of};

use defmt::*;

use ekv::Database;
use embassy_executor::task;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_time::{Duration, Ticker};
use lazy_static::lazy_static;
use matrix::MATRIX_PERIOD;
use reactor::reactor_event::ReactorEvent;
use reactor::Polled;

use embedded_alloc::Heap;

// TODO: Get rid of allocations
#[global_allocator]
static HEAP: Heap = Heap::empty();

use embassy_nrf::interrupt::Priority;
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_nrf::{bind_interrupts, pac, peripherals, qspi, rng, saadc, usb};
use nrf_softdevice::{raw, SocEvent, Softdevice};
use static_cell::make_static;

pub mod analog_nrf;
pub mod ble_hid;
pub mod config;
pub mod config_types;
pub mod data;
pub mod flash_nrf;
pub mod gpio;
pub mod keyboard_report_mid;
pub mod keymap_mid;
pub mod matrix;
pub mod nrf;
pub mod prelude;
pub mod usb_hid;

bind_interrupts!(struct Irqs {
	USBD => usb::InterruptHandler<peripherals::USBD>;
	SAADC => saadc::InterruptHandler;
	QSPI => qspi::InterruptHandler<peripherals::QSPI>;
	RNG => rng::InterruptHandler<peripherals::RNG>;
});

pub type Flash = flash_nrf::Flash<'static>;
pub const PUBSUB_CAPACITY: usize = 20 * size_of::<ReactorEvent>();
pub const PUBSUB_SUBSCRIBERS: usize = 4;
pub const PUBSUB_PUBLISHERS: usize = 4;
pub static CHANNEL: PubSubChannel<
	CriticalSectionRawMutex,
	ReactorEvent,
	PUBSUB_CAPACITY,
	PUBSUB_SUBSCRIBERS,
	PUBSUB_PUBLISHERS,
> = PubSubChannel::new();
lazy_static! {
	// TODO: Add support for HardwareVbusDetect as well to avoid needing the SoftDevice
	pub static ref VBUS_DETECT: SoftwareVbusDetect = SoftwareVbusDetect::new(true, true);
}

pub fn init() -> embassy_nrf::Peripherals {
	info!("PubSubinator v{}", env!("CARGO_PKG_VERSION"));
	{
		use core::mem::MaybeUninit;
		const HEAP_SIZE: usize = 1024;
		static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
		unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
	}

	unsafe {
		let nvmc = &*pac::NVMC::ptr();
		let power = &*pac::POWER::ptr();

		// Enable DC-DC
		power.dcdcen.write(|w| w.dcdcen().enabled());

		// Enable flash cache
		nvmc.icachecnf.write(|w| w.cacheen().enabled());
	}

	let mut config = embassy_nrf::config::Config::default();
	config.gpiote_interrupt_priority = Priority::P2;
	config.time_interrupt_priority = Priority::P2;
	// config.dcdc = DcdcConfig { reg0: false, reg1: true };
	embassy_nrf::init(config)
}

#[task]
pub async fn softdevice_task(sd: &'static Softdevice) {
	info!("SoftDevice task started");
	sd.run_with_callback(|event: SocEvent| {
		info!("SoftDevice event: {:?}", event);

		match event {
			SocEvent::PowerUsbRemoved => VBUS_DETECT.detected(false),
			SocEvent::PowerUsbDetected => VBUS_DETECT.detected(true),
			SocEvent::PowerUsbPowerReady => VBUS_DETECT.ready(),
			_ => {},
		};
	})
	.await;
	info!("SoftDevice task finished");
}

#[task]
pub async fn poller_task(poller: &'static mut dyn Polled) {
	let mut ticker = Ticker::every(Duration::from_millis(MATRIX_PERIOD));
	info!("Poller task started");

	loop {
		// TODO: Turn this into a join of all pollers
		poller.poll().await;
		ticker.next().await;
	}
}

pub fn get_softdevice() -> &'static mut Softdevice {
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
		conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }), // Got from a trace! that said that peer wants 517
		gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t { attr_tab_size: 32768 }),
		gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
			adv_set_count: 1,
			periph_role_count: 3,
			central_role_count: 3,
			central_sec_count: 3,
			_bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
		}),
		gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
			p_value: env!("DEVICE_NAME").as_ptr() as _,
			current_len: env!("DEVICE_NAME").len() as u16,
			max_len: env!("DEVICE_NAME").len() as u16,
			write_perm: unsafe { mem::zeroed() },
			// TODO: Use the SecurityMode enum
			// write_perm: raw::ble_gap_conn_sec_mode_t {
			// 	_bitfield_1: raw::ble_gap_conn_sec_mode_t::new_bitfield_1(1, 4),
			// },
			_bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
		}),
		..Default::default()
	};

	let sd = Softdevice::enable(&sd_config);
	info!("SoftDevice initialized");

	sd
}

pub async fn get_db() -> &'static mut Database<&'static mut flash_nrf::Flash<'static>, CriticalSectionRawMutex> {
	// --- Set the session seed ---
	// TODO: This crashes with `sd_softdevice_enable err SdmIncorrectInterruptConfiguration`
	// let mut rng = embassy_nrf::rng::Rng::new(p.RNG, crate::Irqs);
	// let session_seed = rng.next_u32();
	// TODO: For some reason dropping silently crashes - has to do with the SoftDevice?
	// drop(rng); // Release the RNG as soon as possible

	let flash = make_static!(flash_nrf::Flash::new().await);
	let mut db_config = ekv::Config::default();
	db_config.random_seed = 0xDEADBEEF;
	let db = make_static!(ekv::Database::<_, CriticalSectionRawMutex>::new(flash, db_config));

	#[cfg(feature = "database-format")]
	{
		info!("Formatting EKV Database");
		db.format().await.unwrap();
		info!("EKV Database formatted");
	}

	db
}
