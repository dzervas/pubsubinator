use core::cell::{Cell, RefCell};
use core::pin::Pin;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use ekv::Database;
use embassy_executor::task;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::Subscriber;
use futures::Future;
use heapless::String;
use nrf_softdevice::ble::advertisement_builder::{
	AdvertisementDataType, Flag, LegacyAdvertisementBuilder, ServiceList, ServiceUuid16,
};
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{RegisterError, Service};
use nrf_softdevice::ble::security::SecurityHandler;
use nrf_softdevice::ble::{
	gatt_server, peripheral, Connection, EncryptionInfo, GattValue, IdentityKey, MasterId, Uuid,
};
use nrf_softdevice::Softdevice;
use static_cell::make_static;
use ssmarshal::serialize;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use defmt::*;

use crate::{PUBSUB_CAPACITY, PUBSUB_PUBLISHERS, PUBSUB_SUBSCRIBERS};
use reactor::reactor_event::*;
use reactor::RSubscriber;

#[task]
pub async fn ble_hid_task(sd: &'static Softdevice, server: &'static Server, db: &'static mut Database<&mut crate::Flash, CriticalSectionRawMutex>) {
	info!("BLE HID task started");
	let security_handler = make_static!(Bonder::new(db));

	loop {
		info!("Waiting for connection");
		let conn = BleHid::connect(sd, security_handler).await;

		info!("Got connection: {:?}", conn.peer_address());
		let mut active_conn = server.hid.active_conn_handle.lock().await;
		*active_conn = conn.handle();
		drop(active_conn);
		info!("Updated active connection handle");

		gatt_server::run(&conn, server, |_| {}).await;

		info!("Connection lost");
	}
}

#[nrf_softdevice::gatt_service(uuid = "180f")]
pub struct BatteryService {
	#[characteristic(uuid = "2a19", read, notify)]
	battery_level: u8,
}

// #[repr(packed)]
// pub struct KeyboardReport {
// 	pub modifier: u8,
// 	pub reserved: u8,
// 	pub keycodes: [u8; 6],
// }

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum VidSource {
	BluetoothSIG = 1,
	UsbIF = 2,
}

impl Into<VidSource> for u8 {
	fn into(self) -> VidSource {
		match self {
			1 => VidSource::BluetoothSIG,
			2 => VidSource::UsbIF,
			_ => self::panic!("Invalid VidSource"),
		}
	}
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PnPID {
	pub vid_source: VidSource,
	pub vendor_id: u16,
	pub product_id: u16,
	pub product_version: u16,
}

impl GattValue for PnPID {
	const MAX_SIZE: usize = 7;
	const MIN_SIZE: usize = 7;

	fn from_gatt(data: &[u8]) -> Self {
		Self {
			vid_source: data[0].into(),
			vendor_id: u16::from_le_bytes([data[1], data[2]]),
			product_id: u16::from_le_bytes([data[3], data[4]]),
			product_version: u16::from_le_bytes([data[5], data[6]]),
		}
	}

	fn to_gatt(&self) -> &[u8] {
		// TODO: Find a safe alternative
		unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, core::mem::size_of::<PnPID>()) }
	}
}

#[nrf_softdevice::gatt_service(uuid = "180a")]
pub struct DeviceInformationService {
	#[characteristic(uuid = "2a24", read)]
	model_number: String<32>,

	#[characteristic(uuid = "2a25", read)]
	serial_number: String<32>,

	#[characteristic(uuid = "2a26", read)]
	firmware_revision: String<32>,

	#[characteristic(uuid = "2a27", read)]
	hardware_revision: String<32>,

	#[characteristic(uuid = "2a28", read)]
	software_revision: String<32>,

	#[characteristic(uuid = "2a29", read)]
	manufacturer_name: String<32>,

	#[characteristic(uuid = "2a50", read)]
	pnp_id: PnPID,
}

pub struct HIDService {
	pub hid_info: u16,
	pub report_map: u16,
	pub hid_control: u16,
	pub protocol_mode: u16,
	pub input_keyboard: u16,
	// pub output_keyboard: u16,
	// pub input_media_keys: u16,
	pub active_conn_handle: Arc<Mutex<ThreadModeRawMutex, Option<u16>>>,
}

impl HIDService {
	pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
		let mut service_builder = ServiceBuilder::new(sd, Uuid::new_16(0x1812))?;

		let hid_info = service_builder.add_characteristic(
			Uuid::new_16(0x2A4A),
			Attribute::new([0x11u8, 0x1u8, 0x00u8, 0x01u8]),
			Metadata::new(Properties::new().read()),
		)?;
		let hid_info_handle = hid_info.build();

		let report_map = service_builder.add_characteristic(
			Uuid::new_16(0x2A4B),
			Attribute::new(KeyboardReport::desc()),
			Metadata::new(Properties::new().read()),
		)?;
		let report_map_handle = report_map.build();

		let hid_control = service_builder.add_characteristic(
			Uuid::new_16(0x2A4C),
			Attribute::new([0u8]),
			Metadata::new(Properties::new().write_without_response()),
		)?;
		let hid_control_handle = hid_control.build();

		let mut input_keyboard = service_builder.add_characteristic(
			Uuid::new_16(0x2A4D),
			Attribute::new([0u8; 8]),
			Metadata::new(Properties::new().read().notify()),
		)?;
		let _input_keyboard_desc =
			// TODO: This is the ID of descriptor. Wtf to do with it? let it hardcoded?
			input_keyboard.add_descriptor(Uuid::new_16(0x2908), Attribute::new([1, 1u8]))?; // First is ID (e.g. 1 for keyboard 2 for media keys), second is in/out
		let input_keyboard_handle = input_keyboard.build();

		// TODO: Handle outputs

		let protocol_mode = service_builder.add_characteristic(
			Uuid::new_16(0x2A4E),
			Attribute::new([1u8]),
			Metadata::new(Properties::new().read().write_without_response()),
		)?;
		let protocol_mode_handle = protocol_mode.build();

		let _service_handle = service_builder.build();

		Ok(HIDService {
			hid_info: hid_info_handle.value_handle,
			report_map: report_map_handle.value_handle,
			hid_control: hid_control_handle.value_handle,
			protocol_mode: protocol_mode_handle.value_handle,
			input_keyboard: input_keyboard_handle.value_handle,
			active_conn_handle: Arc::new(Mutex::new(None)),
		})
	}

	pub async fn send_report(&self, report: &KeyboardReport) {
		let active_conn = self.active_conn_handle.lock().await;
		if active_conn.is_none() {
			info!("No active connection");
			return;
		}

		let conn = Connection::from_handle(active_conn.unwrap()).unwrap();
		drop(active_conn);

		// TODO: The size needs to be a generic or const
		let mut report_bytes = [0u8; 9];
		serialize(&mut report_bytes, report).expect("Failed to serialize report");

		match gatt_server::notify_value(&conn, self.input_keyboard, &report_bytes) {
			Ok(_) => {},
			Err(e) => warn!("Error sending BLE HID report: {:?}", e),
		}
	}
}
type HIDServiceEvent = ();

impl Service for HIDService {
	type Event = HIDServiceEvent;
	fn on_write(&self, handle: u16, data: &[u8]) -> Option<Self::Event> {
		info!("HIDService::on_write: handle: {:x}, data: {:?}", handle, data);
		None
	}
}

#[nrf_softdevice::gatt_server]
pub struct Server {
	pub bas: BatteryService,
	pub hid: HIDService,
	pub dis: DeviceInformationService,
}

impl Server {
	pub fn init(&mut self) {
		self.dis
			.model_number_set(&String::try_from(env!("DEVICE_NAME")).unwrap())
			.unwrap();
		self.dis
			.serial_number_set(&String::try_from(env!("DEVICE_SERIAL")).unwrap())
			.unwrap();
		self.dis
			.manufacturer_name_set(&String::try_from("PubSubinator").unwrap())
			.unwrap();
		self.dis
			.pnp_id_set(&PnPID {
				vid_source: VidSource::UsbIF,
				vendor_id: 0xC0DE,
				product_id: 0xCAFE,
				product_version: u16::from_str_radix(env!("DEVICE_VERSION").replace(".", "").as_str(), 16)
					.expect("Device version is not a valid hex number"),
			})
			.unwrap();

		self.bas.battery_level_set(&66).unwrap();
	}
}

pub struct BleHid<'a> {
	pub softdevice: &'a Softdevice,
	pub server: &'a Server,
	pub channel:
		Subscriber<'a, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>,
}

impl<'a> BleHid<'a> {
	pub async fn connect(sd: &'a Softdevice, security_handler: &'static dyn SecurityHandler) -> Connection {
		let adv_data = LegacyAdvertisementBuilder::new()
			.flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
			.services_16(
				ServiceList::Incomplete,
				&[ServiceUuid16::BATTERY, ServiceUuid16::HUMAN_INTERFACE_DEVICE],
			)
			.full_name(env!("DEVICE_NAME"))
			.raw(AdvertisementDataType::APPEARANCE, &[0xC1, 0x03])
			.build();

		let scan_data = LegacyAdvertisementBuilder::new()
			.services_16(
				ServiceList::Complete,
				&[
					ServiceUuid16::BATTERY,
					ServiceUuid16::DEVICE_INFORMATION,
					ServiceUuid16::HUMAN_INTERFACE_DEVICE,
				],
			)
			.build();

		let config = peripheral::Config::default();
		let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
			adv_data: &adv_data,
			scan_data: &scan_data,
		};

		info!("advertising...");

		let conn = peripheral::advertise_pairable(sd, adv, &config, security_handler)
			.await
			.unwrap();
		info!("Updating active connection handle");

		info!("advertising done!");

		conn
	}
}

impl<'a> RSubscriber for BleHid<'a> {
	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async move {
			match value {
				ReactorEvent::KeyboardReport { modifier, keycodes } => {
					let report = KeyboardReport {
						modifier: modifier.into(),
						reserved: 0,
						leds: 0,
						// TODO: Make this a generic
						keycodes: [
							keycodes[0].into(),
							keycodes[1].into(),
							keycodes[2].into(),
							keycodes[3].into(),
							keycodes[4].into(),
							keycodes[5].into(),
						],
					};

					self.server.hid.send_report(&report).await;
				},
				_ => {},
			}
		})
	}
}

#[derive(Debug, Clone, Copy)]
struct Peer {
	master_id: MasterId,
	key: EncryptionInfo,
	peer_id: IdentityKey,
}

pub struct Bonder {
	peer: Cell<Option<Peer>>,
	sys_attrs: RefCell<Vec<u8>>,
	db: &'static mut Database<&'static mut crate::Flash, CriticalSectionRawMutex>,
}

impl Bonder {
	pub fn new(db: &'static mut Database<&'static mut crate::Flash, CriticalSectionRawMutex>) -> Self {
		Bonder {
			peer: Cell::new(None),
			sys_attrs: Default::default(),
			db,
		}
	}
}

impl SecurityHandler for Bonder {
	fn io_capabilities(&self) -> nrf_softdevice::ble::security::IoCapabilities {
		nrf_softdevice::ble::security::IoCapabilities::None
	}

	// fn can_recv_out_of_band(&self, _conn: &nrf_softdevice::ble::Connection) -> bool {
	// 	true
	// }

	fn can_bond(&self, _conn: &nrf_softdevice::ble::Connection) -> bool {
		true
	}

	fn display_passkey(&self, passkey: &[u8; 6]) {
		info!("display_passkey {:?}", passkey);
	}

	// fn enter_passkey(&self, _reply: nrf_softdevice::ble::PasskeyReply) {
	// 	info!("enter_passkey");
	// }

	// fn recv_out_of_band(&self, _reply: nrf_softdevice::ble::OutOfBandReply) {
	// 	info!("recv_out_of_band");
	// }

	fn on_security_update(
		&self,
		_conn: &nrf_softdevice::ble::Connection,
		security_mode: nrf_softdevice::ble::SecurityMode,
	) {
		info!("on_security_update {:?}", security_mode);
	}

	fn on_bonded(
		&self,
		_conn: &nrf_softdevice::ble::Connection,
		master_id: nrf_softdevice::ble::MasterId,
		key: nrf_softdevice::ble::EncryptionInfo,
		peer_id: nrf_softdevice::ble::IdentityKey,
	) {
		info!("on_bonded");

		// In a real application you would want to signal another task to permanently store the keys in non-volatile memory here.
		self.sys_attrs.borrow_mut().clear();
		let peer = Peer {
			master_id,
			key,
			peer_id,
		};
		self.peer.set(Some(peer));
	}

	fn get_key(&self, _conn: &nrf_softdevice::ble::Connection, master_id: MasterId) -> Option<EncryptionInfo> {
		debug!("getting bond for: id: {}", master_id);

		self.peer
			.get()
			.and_then(|peer| (master_id == peer.master_id).then_some(peer.key))
	}

	fn save_sys_attrs(&self, conn: &nrf_softdevice::ble::Connection) {
		debug!("saving system attributes for: {}", conn.peer_address());

		if let Some(peer) = self.peer.get() {
			if peer.peer_id.is_match(conn.peer_address()) {
				let mut sys_attrs = self.sys_attrs.borrow_mut();
				let capacity = sys_attrs.capacity();
				sys_attrs.resize(capacity, 0);
				if let Ok(len) = gatt_server::get_sys_attrs(conn, &mut sys_attrs) {
					sys_attrs.truncate(len);
					// In a real application you would want to signal another task to permanently store sys_attrs for this connection's peer
				}
			}
		}
	}

	fn load_sys_attrs(&self, conn: &nrf_softdevice::ble::Connection) {
		let addr = conn.peer_address();
		debug!("loading system attributes for: {}", addr);

		let attrs = self.sys_attrs.borrow();
		// In a real application you would search all stored peers to find a match
		let attrs = if self.peer.get().map(|peer| peer.peer_id.is_match(addr)).unwrap_or(false) {
			(!attrs.is_empty()).then_some(attrs.as_slice())
		} else {
			None
		};

		gatt_server::set_sys_attrs(conn, attrs).unwrap();
	}
}
