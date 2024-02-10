// We need to use alloc Vec, otherwise we can instatiate the KeymapConfig
// without generics
use alloc::vec::Vec;
use core::str::FromStr;
use reactor::*;

use crate::keymap_mid::*;

#[derive(Debug, Default)]
pub struct KeymapConfig {
	pub layers: Vec<Vec<Vec<&'static str>>>,
	pub hold_cycles: u16,
}

impl KeymapConfig {
	pub fn build(&self) -> Keymap {
		let layers = self.layers.iter().map(|layer| {
			layer.iter().map(|row| {
				row.iter().map(|key| {
					KeyCodeInt::Key(KeyCode::from_str(key).unwrap())
				}).collect::<Vec<KeyCodeInt>>()
			}).collect::<Vec<Vec<KeyCodeInt>>>()
		}).collect::<Vec<Vec<Vec<KeyCodeInt>>>>();

		Keymap::new(layers, self.hold_cycles)
	}

}
