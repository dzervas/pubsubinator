#![no_std]
#![no_main]
// make_static! macro requires this
#![feature(type_alias_impl_trait)]

use pubsubinator::prelude::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = init();

	// --- Setup Matrix publisher ---
	let matrix = make_static!(config::MATRIX.build());
	spawner.spawn(poller_task(matrix)).unwrap();
	info!("Matrix publisher initialized");

	// --- Setup Keymap middleware ---
	let keymap = make_static!(config::KEYMAP.build());
	info!("Keymap middleware initialized");

	// --- Setup Keyboard Report middleware ---
	let keyboard_report = make_static!(keyboard_report_mid::KeyboardReportMid::default());

	// --- Setup Analog publisher ---

	let analog = make_static!(Analog::new(p.SAADC, [
		Into::<saadc::AnyInput>::into(p.P0_03),
		Into::<saadc::AnyInput>::into(p.P0_04),
	]));
	spawner.spawn(poller_task(analog)).unwrap();

	// --- Setup USB HID consumer ---
	let mut usb_builder = usb_init(p.USBD);
	let usb_hid = make_static!(UsbHid::new(&mut usb_builder));

	spawner.spawn(usb_task(usb_builder)).unwrap();
	info!("USB HID consumer initialized");

	// --- Setup SoftDevice ---
	let sd = get_softdevice();

	// --- Setup EKV DB ---
	let db = get_db().await;

	let server = make_static!(ble_hid::Server::new(sd).unwrap());
	server.init();

	// --- Setup BLE HID consumer ---
	let ble_hid = make_static!(BleHid {
		softdevice: sd,
		server,
		channel: CHANNEL.subscriber().unwrap(),
	});

	spawner.spawn(softdevice_task(sd)).unwrap();
	spawner.spawn(ble_hid_task(sd, server, db)).unwrap();

	let subs_task = subscribers_task_env!(CHANNEL, "PUBSUB_SUBSCRIBERS", "PUBSUB_MIDDLEWARE");
	spawner.spawn(subs_task).unwrap();
}
