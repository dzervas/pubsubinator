use core::pin::Pin;

use alloc::boxed::Box;
use defmt::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use futures::Future;

use crate::{CHANNEL, PUBSUB_CAPACITY, PUBSUB_PUBLISHERS, PUBSUB_SUBSCRIBERS};
use reactor::middleware::Middleware;
use reactor::reactor_event::*;

pub const KEYMAP_PERIOD: u64 = 2;

pub struct Keymap<const R: usize, const C: usize, const L: usize> {
	keymap: [[[KeyCodeInt; C]; R]; L],
	pub hold_cycles: u16,
	layer: usize,
	last_state: [[(KeyEvent, u8); C]; R],
	channel: Publisher<
		'static,
		CriticalSectionRawMutex,
		ReactorEvent,
		PUBSUB_CAPACITY,
		PUBSUB_SUBSCRIBERS,
		PUBSUB_PUBLISHERS,
	>,
}

impl<const R: usize, const C: usize, const L: usize> Keymap<R, C, L> {
	pub fn new(keymap: [[[KeyCodeInt; C]; R]; L], hold_cycles: u16) -> Self {
		let last_state = [[(KeyEvent::Released(KeyCode::None), 0); C]; R];

		Self {
			keymap,
			hold_cycles,
			last_state,
			layer: 0,
			channel: CHANNEL.publisher().unwrap(),
		}
	}
}

// TODO: Specify the is_supported
impl<const R: usize, const C: usize, const L: usize> Middleware for Keymap<R, C, L> {
	fn process(&mut self, event: ReactorEvent) -> Pin<Box<dyn Future<Output = Option<ReactorEvent>> + '_>> {
		Box::pin(async move {
			let (value, rindex, cindex) = match event {
				ReactorEvent::HardwareMappedBool(value, rindex, cindex) => (value, rindex, cindex),
				_ => return None,
			};
			let active_keymap = &self.keymap[self.layer];

			let mut new_state: KeyEvent = self.last_state[rindex][cindex].0;

			if let KeyCodeInt::Internal(event) = active_keymap[rindex][cindex] {
				let old_layer = self.layer;
				match event {
					InternalEvent::LayerNext => {
						self.layer += 1;
						if self.layer >= L {
							self.layer = 0;
						}
					},
					InternalEvent::LayerPrev =>
						if self.layer == 0 {
							self.layer = L - 1;
						} else {
							self.layer -= 1;
						},
					InternalEvent::LayerChange(target) => {
						self.layer = target;
						if self.layer >= L {
							self.layer = 0;
						}
					},
					_ => {},
				}

				if old_layer != self.layer {
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
