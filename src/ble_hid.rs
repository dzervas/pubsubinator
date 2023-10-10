use core::cell::{Cell, RefCell};
use core::pin::Pin;

use alloc::boxed::Box;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Subscriber;
use futures::Future;
use heapless::String;
use nrf_softdevice::ble::gatt_server::GetValueError;
use nrf_softdevice::ble::security::SecurityHandler;
use nrf_softdevice::ble::{gatt_server, peripheral, MasterId, EncryptionInfo, IdentityKey, GattValue};
use nrf_softdevice::{raw, Softdevice};
use usbd_hid::descriptor::KeyboardReport;

use defmt::*;

use reactor::RSubscriber;
use reactor::reactor_event::*;
use crate::{PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS};

#[nrf_softdevice::gatt_service(uuid = "180f")]
pub struct BatteryService {
	#[characteristic(uuid = "2a19", read, notify)]
	battery_level: u8,
}

// pub struct KeyboardReport {
// 	pub modifier: u8,
// 	pub reserved: u8,
// 	pub leds: u8,
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
		unsafe {
			core::slice::from_raw_parts(
				self as *const Self as *const u8,
				core::mem::size_of::<PnPID>(),
			)
		}
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

#[nrf_softdevice::gatt_service(uuid = "1812")]
pub struct KeyboardService {
	#[characteristic(uuid = "2a4a", read)]
	hid_information: [u8; 4], // Typically, the HID Information is 4 bytes, but it can vary.

	#[characteristic(uuid = "2a4c", write_without_response)]
	hid_control_point: u8, // HID Control Point, used for control commands like "suspend" or "exit suspend".

	// Boot Keyboard Input Report
	// #[characteristic(uuid = "2a22", read, notify, getter="input_getter")]
	#[characteristic(uuid = "2a22", read, notify)]
	boot_input: [u8; 8], // This size can vary based on your needs.

	// Boot Keyboard Output Report (LED states like Caps Lock)
	#[characteristic(uuid = "2a32", read, write, write_without_response)]
	boot_output: u8,

	// HID Report Map (describes the format of the HID reports)
	#[characteristic(uuid = "2a4b", read)]
	// report_map: [u8; 47], // This will be an array that describes the report format.
	report_map: [u8; 63], // This will be an array that describes the report format.

	#[characteristic(uuid = "2a4d", read, write, notify, getter="input_getter")]
	// #[characteristic(uuid = "2a4d", read, write, notify)]
	report: [u8; 8], // This is the actual report data.

	#[characteristic(uuid = "2a4e", read, write_without_response)]
	protocol_mode: u8, // Protocol Mode (boot/report mode)
}

impl KeyboardService {
	pub fn input_getter(&self, _sd: &Softdevice, _value_handle: u16) -> Result<[u8; 8], GetValueError> {
		info!("---> input_getter!!");
		Ok([0, 0, 4, 5, 6, 7, 8, 9])
	}
}

// pub struct HidReportAttribute<T: AsRef<[u8]>>(Attribute<T>);

// impl<T: AsRef<[u8]>> HidReportAttribute<T> {
// 	pub fn new(value: T) -> Self {
// 		Self(Attribute::new(value))
// 	}

// 	pub fn deferred_read(&self) -> Attribute<T> {
// 		info!("deferred_read");
// 		self.0.deferred_read()
// 	}

// 	pub fn read_security(self, security: nrf_softdevice::ble::SecurityMode) -> Self {
// 		info!("read_security");
// 		Self(self.0.read_security(security))
// 	}
// }

#[nrf_softdevice::gatt_server]
pub struct Server {
	bas: BatteryService,
	keyboard: KeyboardService,
	device_information: DeviceInformationService,
}

pub struct BleHid<'a> {
	pub softdevice: &'a Softdevice,
	pub server: Server,
	pub report: KeyboardReport,
	pub channel: Subscriber<'a, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>,
	pub security_handler: &'static Bonder,
}

impl<'a> BleHid<'a> {
	pub async fn run(&mut self) {
		#[rustfmt::skip]
		let adv_data = &[
			0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
			// 0x03, 0x03, 0x0f, 0x18, // 3 bytes wide, Battery service
			// 0x03, 0x03, 0x12, 0x18, // 3 bytes wide, HID service
			// 0x0a, 0x0c, b"PubSubinator"[..],
			// 0x0a, 0x0c, b'P', b'u', b'b', b'S', b'u', b'b', b'i', b'n', b'a', b't', b'o', b'r',
			0x0a, 0x09, b'P', b'u', b'b', b'S', b'u', b'b', b'i', b'n', b'a',
		];
		#[rustfmt::skip]
		let scan_data = &[
			0x03, 0x03, 0x0f, 0x18, // 3 bytes wide, Battery service
			0x03, 0x03, 0x12, 0x18, // 3 bytes wide, HID service
		];

		self.server.device_information.model_number_set(&String::from("Launchpad")).unwrap();
		self.server.device_information.serial_number_set(&String::from("123456")).unwrap();
		self.server.device_information.manufacturer_name_set(&String::from("PubSubinator")).unwrap();
		self.server.device_information.pnp_id_set(&PnPID {
			vid_source: VidSource::UsbIF,
			vendor_id: 0x10C4,
			product_id: 0x0001,
			product_version: 0x0001,
		}).unwrap();
		// self.server.device_information.pnp_id.vendor_id_set(&0x0002).unwrap();
		// self.server.device_information.pnp_id.product_version_set(&0x0003).unwrap();

		self.server.keyboard.report_set(&[0x12, 0x34, 0xac, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
		self.server.keyboard.hid_information_set(&[
			0x11, 0x01, // bcdHID (USB HID version)
			0x00, // bCountryCode
			0x02, // Flags
				  // Bit 0: Indicates if the device is capable of sending a wake signal to a host. If it can, this bit is set to 1, otherwise 0.
				  // Bit 1: Indicates if the device is in Normally Connectable mode (0) or not (1).
				  // Bit 2: This flag indicates if the device supports Boot Protocol Mode. For a Boot Keyboard, this bit should be set to 1.
		]).unwrap();

		self.server.keyboard.report_map_set(&[
			// Report map
			0x05, 0x01, // Usage Page (Generic Desktop)
			0x09, 0x06, // Usage (Keyboard)
			0xa1, 0x01, // Collection (Application)
			0x85, 0x01, // Report ID (1)
			0x05, 0x07, // Usage Page (Keyboard)
			0x19, 0xe0, // Usage Minimum (Keyboard LeftControl)
			0x29, 0xe7, // Usage Maximum (Keyboard Right GUI)
			0x15, 0x00, // Logical Minimum (0)
			0x25, 0x01, // Logical Maximum (1)
			0x95, 0x08, // Report Count (8)
			0x75, 0x01, // Report Size (1)
			0x81, 0x02, // Input (Data, Variable, Absolute) Modifier byte
			0x95, 0x01, // Report Count (1)
			0x75, 0x08, // Report Size (8)
			0x81, 0x01, // Input (Constant, Array, Absolute) Reserved byte
			0x95, 0x06, // Report Count (6)
			0x75, 0x08, // Report Size (8)
			0x15, 0x00, // Logical Minimum (0)
			0x25, 0x65, // Logical Maximum (101)
			0x19, 0x00, // Usage Minimum (0)
			0x29, 0x65, // Usage Maximum (101)
			0x81, 0x01, // Input (Data, Array, Absolute) Reserved byte
			0x95, 0x05, // Report Count (5) - Num lock, Caps lock, Scroll lock, Compose, Kana
			0x75, 0x01, // Report Size (1)
			0x05, 0x08, // Usage Page (LEDs)
			0x19, 0x01, // Usage Minimum (Num Lock)
			0x29, 0x05, // Usage Maximum (Kana)
			0x91, 0x01, // Output (Data, Variable, Absolute) LED report
			0x95, 0x01, // Report Count (1)
			0x75, 0x03, // Report Size (3)
			0x91, 0x01, // Output (Data, Variable, Absolute) LED report padding
			// 0x05, 0x07, // Usage Page (Key Codes)
			// 0x05, 0x01, // Usage Minimum (Reserved (no event indicated))
			// 0x05, 0x01, // Usage Maximum (Keyboard Application)
			// 0x05, 0x01, // Input (Data,Array) Key arrays (6 bytes)
			0xc0, // End Collection
			// 0x00, 0x00, 0x00
		]).unwrap();

		self.server.keyboard.protocol_mode_set(&0x01).unwrap();

		// self.server.keyboard.boot_input_set(&[0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
		self.server.bas.battery_level_set(&66).unwrap();

		loop {
			let config = peripheral::Config::default();
			let adv = peripheral::ConnectableAdvertisement::ScannableUndirected { adv_data, scan_data };
			// let conn = peripheral::advertise_connectable(self.softdevice, adv, &config).await.unwrap();
			let conn = peripheral::advertise_pairable(self.softdevice, adv, &config, self.security_handler).await.unwrap();

			info!("advertising done!");

			// Run the GATT server on the connection. This returns when the connection gets disconnected.
			//
			// Event enums (ServerEvent's) are generated by nrf_softdevice::gatt_server
			// proc macro when applied to the Server struct above
			let e = gatt_server::run(&conn, &self.server, |e| match e {
				ServerEvent::Bas(e) => match e {
					BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
						info!("battery notifications: {}", notifications)
					}
				},
				ServerEvent::Keyboard(e) => match e {
					KeyboardServiceEvent::HidControlPointWrite(d) => {
						info!("HID control point: {:?}", d)
					},
					KeyboardServiceEvent::BootInputCccdWrite { notifications } => {
						info!("boot input notifications: {}", notifications)
					},
					KeyboardServiceEvent::BootOutputWrite(d) => {
						info!("boot output: {:?}", d)
					},
					KeyboardServiceEvent::ProtocolModeWrite(d) => {
						info!("protocol mode: {:?}", d);
						self.server.keyboard.protocol_mode_set(&0x00).unwrap();
					},
					KeyboardServiceEvent::ReportWrite(d) => {
						info!("report: {:?}", d);
					},
					KeyboardServiceEvent::ReportCccdWrite { notifications } => {
						info!("report notifications: {}", notifications)
					},
				},
				ServerEvent::DeviceInformation(e) => match e {

				}
			})
			.await;

			info!("gatt_server run exited with error: {:?}", e);
		}
	}
}

impl<'a> RSubscriber for BleHid<'a> {
	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async move {
			match value {
				ReactorEvent::Key(code) => {
					match code {
						KeyEvent::Pressed(key) => {
							info!("Pressed: {:?}", key);
							if key > KeyCode::LCtrl && key < KeyCode::RGui {
								self.report.modifier |= 1 << (key as u8 - KeyCode::LCtrl as u8);
							} else if !self.report.keycodes.contains(&(key as u8)) {
								if let Some(pos) = self.report.keycodes.iter().position(|&k| k == KeyCode::None as u8) {
									self.report.keycodes[pos] = key as u8;
								}
							}
						},
						KeyEvent::Released(key) => {
							info!("Released: {:?}", key);
							if key > KeyCode::LCtrl && key < KeyCode::RGui {
								self.report.modifier &= 0 << (key as u8 - KeyCode::LCtrl as u8);
							} else if let Some(pos) = self.report.keycodes.iter().position(|&k| k == key as u8) {
								self.report.keycodes[pos] = 0;
							}
						},
						// _ => {
						// 	info!("Unhandled event: {:?}", value);
						// },
					}
				},
				// ReactorEvent::Locks { caps, num, scroll } => {
				// 	self.report.modifier = 0;
				// 	self.report.keycodes[0] = caps as u8;
				// },
				// ReactorEvent::Mouse { x, y } => {
				// 	info!("Unhandled event: {:?}", value);
				// },
				_ => {
					info!("Unhandled event: {:?}", value);
					return;
				},
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
}

impl Default for Bonder {
	fn default() -> Self {
		Bonder {
			peer: Cell::new(None),
			sys_attrs: Default::default(),
		}
	}
}

impl SecurityHandler for Bonder {
	fn io_capabilities(&self) -> nrf_softdevice::ble::security::IoCapabilities {
		nrf_softdevice::ble::security::IoCapabilities::DisplayOnly
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

	fn on_security_update(&self, _conn: &nrf_softdevice::ble::Connection, security_mode: nrf_softdevice::ble::SecurityMode) {
		info!("on_security_update {:?}", security_mode);
	}

	fn on_bonded(&self, _conn: &nrf_softdevice::ble::Connection, master_id: nrf_softdevice::ble::MasterId, key: nrf_softdevice::ble::EncryptionInfo, peer_id: nrf_softdevice::ble::IdentityKey) {
		info!("on_bonded");

		// In a real application you would want to signal another task to permanently store the keys in non-volatile memory here.
		self.sys_attrs.borrow_mut().clear();
		self.peer.set(Some(Peer {
			master_id,
			key,
			peer_id,
		}));
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
				let len = unwrap!(gatt_server::get_sys_attrs(conn, &mut sys_attrs)) as u16;
				sys_attrs.truncate(usize::from(len));
				// In a real application you would want to signal another task to permanently store sys_attrs for this connection's peer
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
