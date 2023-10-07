use core::pin::Pin;

use alloc::boxed::Box;
use alloc::vec::Vec;
use defmt::*;
use futures::Future;

use crate::middleware::Middleware;
use crate::reactor_event::{ReactorEvent, KeyCode, KeyEvent};

pub const KEYMAP_PERIOD: u64 = 2;

pub struct Keymap {
	keymap: Vec<Vec<KeyCode>>,
	pub debounce_cycles: u8,
	pub hold_cycles: u16,
	last_state: Vec<Vec<(KeyEvent, u8)>>,
}

impl Keymap {
	pub fn new(keymap: Vec<Vec<KeyCode>>, debounce_cycles: u8, hold_cycles: u16) -> Self {
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
			debounce_cycles,
			hold_cycles,
			last_state,
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
					if value && self.last_state[rindex][cindex].1 >= self.debounce_cycles {
						info!("Got a pressed event: {:?}", &code);
						new_state = KeyEvent::Pressed(code);
					}
				},
				KeyEvent::Pressed(code) => {
					if value {
						info!("Got a released event: {:?}", &code);
						new_state = KeyEvent::Released(code)
					}
				},
			};

			if new_state != self.last_state[rindex][cindex].0 {
				self.last_state[rindex][cindex].0 = new_state;
				self.last_state[rindex][cindex].1 = 0;
				return Some(ReactorEvent::Key(new_state));
			}

			// Integer overload should be handled as a normal event
			self.last_state[rindex][cindex].1 += 1;

			None
		})
	}
}
