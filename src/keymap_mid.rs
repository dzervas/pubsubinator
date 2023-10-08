use core::pin::Pin;

use alloc::boxed::Box;
use defmt::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use futures::Future;

use reactor::middleware::Middleware;
use reactor::reactor_event::*;
use crate::{CHANNEL, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS};

pub const KEYMAP_PERIOD: u64 = 2;

pub struct Keymap<const R: usize, const C: usize> {
	keymap: [[KeyCode; C]; R],
	pub hold_cycles: u16,
	last_state: [[(KeyEvent, u8); C]; R],
	channel: Publisher<'static, CriticalSectionRawMutex, ReactorEvent, PUBSUB_CAPACITY, PUBSUB_SUBSCRIBERS, PUBSUB_PUBLISHERS>
}

impl<const R: usize, const C: usize> Keymap<R, C> {
	pub fn new(keymap: [[KeyCode; C]; R], hold_cycles: u16) -> Self {
		let last_state = [[(KeyEvent::Released(KeyCode::None), 0); C]; R];

		Self {
			keymap,
			hold_cycles,
			last_state,
			channel: CHANNEL.publisher().unwrap()
		}
	}
}

// TODO: Specify the is_supported
impl<const R: usize, const C: usize> Middleware for Keymap<R, C> {
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
