use core::pin::Pin;

use alloc::boxed::Box;
use alloc::vec::Vec;
use defmt::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use futures::Future;

use reactor::middleware::Middleware;
use reactor::reactor_event::*;
use crate::{CHANNEL, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS};

pub const KEYMAP_PERIOD: u64 = 2;

pub struct Keymap {
	keymap: Vec<Vec<KeyCode>>,
	pub hold_cycles: u16,
	last_state: Vec<Vec<(KeyEvent, u8)>>,
	channel: Publisher<'static, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>
}

impl Keymap {
	pub fn new(keymap: Vec<Vec<KeyCode>>, hold_cycles: u16) -> Self {
		let mut last_state = Vec::new();

		for r in keymap.iter() {
			let mut row = Vec::new();
			for c in r.iter() {
				row.push((KeyEvent::Released(c.clone()), 0));
			}
			last_state.push(row);
		}

		Self {
			keymap,
			hold_cycles,
			last_state,
			channel: CHANNEL.publisher().unwrap()
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

			let mut new_state: KeyEvent = self.last_state[rindex][cindex].0;

			match self.last_state[rindex][cindex].0 {
				KeyEvent::Released(code) => {
					if value {
						info!("Got a pressed event: {:?}", &code);
						new_state = KeyEvent::Pressed(self.keymap[rindex][cindex].clone());
					}
				},
				KeyEvent::Pressed(code) => {
					if !value {
						info!("Got a released event: {:?}", &code);
						new_state = KeyEvent::Released(self.keymap[rindex][cindex].clone())
					}
				},
			};

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
