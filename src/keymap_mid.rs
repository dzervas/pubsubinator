use core::pin::Pin;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use defmt::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use futures::Future;

use crate::{CHANNEL, PUBSUB_CAPACITY, PUBSUB_PUBLISHERS, PUBSUB_SUBSCRIBERS};
use reactor::middleware::Middleware;
use reactor::reactor_event::*;

pub const KEYMAP_PERIOD: u64 = 2;

pub struct Keymap {
	pub layers: Vec<Vec<Vec<KeyCodeInt>>>,
	pub hold_cycles: u16,
	current_layer: usize,
	last_state: Vec<Vec<(KeyEvent, u8)>>,
	channel: Publisher<
		'static,
		CriticalSectionRawMutex,
		ReactorEvent,
		PUBSUB_CAPACITY,
		PUBSUB_SUBSCRIBERS,
		PUBSUB_PUBLISHERS,
	>,
}

impl Keymap {
	pub fn new(keymap: Vec<Vec<Vec<KeyCodeInt>>>, hold_cycles: u16) -> Self {
		let last_state = vec![vec![(KeyEvent::Released(KeyCode::None), 0); keymap[0][0].len()]; keymap[0].len()];
		Self {
			layers: keymap,
			hold_cycles,
			last_state,
			..Default::default()
		}
	}
}

impl Default for Keymap {
	fn default() -> Self {
		let keymap = vec![vec![vec![]]];

		Self {
			layers: keymap,
			hold_cycles: 0,
			last_state: vec![vec![(KeyEvent::Released(KeyCode::None), 0)]],
			current_layer: 0,
			channel: CHANNEL.publisher().unwrap(),
		}
	}
}

// TODO: Specify the is_supported
impl Middleware for Keymap {
	fn process(&mut self, event: ReactorEvent) -> Pin<Box<dyn Future<Output = Option<ReactorEvent>> + '_>> {
		Box::pin(async move {
			let (value, rindex, cindex) = match event {
				ReactorEvent::HardwareMappedBool(value, rindex, cindex) => (value, rindex, cindex),
				_ => return None,
			};
			let active_keymap = &self.layers[self.current_layer];

			let mut new_state: KeyEvent = self.last_state[rindex][cindex].0;

			if let KeyCodeInt::Internal(event) = active_keymap[rindex][cindex] {
				let old_layer = self.current_layer;
				match event {
					InternalEvent::LayerNext => {
						self.current_layer += 1;
						if self.current_layer >= self.layers.len() {
							self.current_layer = 0;
						}
					},
					InternalEvent::LayerPrev =>
						if self.current_layer == 0 {
							self.current_layer = self.layers.len() - 1;
						} else {
							self.current_layer -= 1;
						},
					InternalEvent::LayerChange(target) => {
						self.current_layer = target;
						if self.current_layer >= self.layers.len() {
							self.current_layer = 0;
						}
					},
					_ => {},
				}

				if old_layer != self.current_layer {
					for row in self.last_state.iter_mut() {
						for code in row.iter_mut() {
							if let KeyEvent::Pressed(key) = code.0 {
								code.0 = KeyEvent::Released(key);
								code.1 = 0;
								self.channel.publish(ReactorEvent::Key(KeyEvent::Released(key))).await;
							}
						}
					}
				}
			} else if let KeyCodeInt::Key(key) = active_keymap[rindex][cindex] {
				match self.last_state[rindex][cindex].0 {
					KeyEvent::Released(code) =>
						if value {
							info!("Got a pressed event: {:?}", &code);
							new_state = KeyEvent::Pressed(key.clone());
						},
					KeyEvent::Pressed(code) =>
						if !value {
							info!("Got a released event: {:?}", &code);
							new_state = KeyEvent::Released(key.clone())
						},
				};
			}

			if new_state != self.last_state[rindex][cindex].0 {
				self.last_state[rindex][cindex].0 = new_state;
				self.last_state[rindex][cindex].1 = 0;
				self.channel.publish(ReactorEvent::Key(new_state)).await;
				return Some(ReactorEvent::Key(new_state));
			}

			// Integer overload should be handled as a normal event
			self.last_state[rindex][cindex].1 += 1;

			None
		})
	}
}
