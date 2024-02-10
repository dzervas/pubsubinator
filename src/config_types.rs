// We need to use alloc Vec, otherwise we can instatiate the KeymapConfig
// without generics
use alloc::vec::Vec;
use core::str::FromStr;
use reactor::*;

use crate::{gpio::{Drive, Input, Level, Output, Pin, Pull}, keymap_mid::*, matrix::{InputObj, Matrix, MatrixDirection, OutputObj}};

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

#[derive(Debug, Default)]
pub struct MatrixConfig {
	pub inputs: Vec<MatrixConfigInputsType>,
	pub outputs: Vec<MatrixConfigOutputsType>,
	pub direction: &'static str,
}

impl MatrixConfig {
	pub fn build(&self) -> Matrix<Input, Output> {
		let inputs = self.inputs.iter().map(|input| {
			input.into()
		}).collect::<Vec<Input>>();
		let outputs = self.outputs.iter().map(|output| {
			output.into()
		}).collect::<Vec<Output>>();

		Matrix::new(inputs, outputs, MatrixDirection::from_str(self.direction).unwrap())
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MatrixConfigInputsType {
	pub pin: &'static str,
	pub pull: &'static str,
}

impl<'a> Into<Input<'a>> for &MatrixConfigInputsType {
	fn into(self) -> Input<'a> {
		let pin = Pin::from_str(self.pin).unwrap();
		let pull = Pull::from_str(self.pull).unwrap();
		Input::new(pin, pull.into())
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MatrixConfigOutputsType {
	pub pin: &'static str,
	pub drive: &'static str,
	pub level: &'static str,
}

impl<'a> Into<Output<'a>> for &MatrixConfigOutputsType {
	fn into(self) -> Output<'a> {
		let pin = Pin::from_str(self.pin).unwrap();
		let drive = Drive::from_str(self.drive).unwrap();
		let level = Level::from_str(self.level).unwrap();
		Output::new(pin, level.into(), drive.into())
	}
}
