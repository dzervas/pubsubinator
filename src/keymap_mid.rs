use alloc::vec::Vec;
use defmt::*;

use crate::middleware::PublisherMiddleware;
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

impl PublisherMiddleware<Vec<Vec<bool>>> for Keymap {
	async fn process(&mut self, value_map: Vec<Vec<bool>>) -> Option<ReactorEvent> {
		for (rindex, rarray) in value_map.iter().enumerate() {
			for (cindex, cvalue) in rarray.iter().enumerate() {
				let mut new_state: KeyEvent = self.last_state[rindex][cindex].0;

				match self.last_state[rindex][cindex].0 {
					KeyEvent::Released(code) => {
						if *cvalue && self.last_state[rindex][cindex].1 >= self.debounce_cycles {
							info!("Got a pressed event: {:?}", &code);
							new_state = KeyEvent::Pressed(code);
						}
					},
					KeyEvent::Pressed(code) => {
						if !*cvalue {
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
			}
		}

		None
	}
}
