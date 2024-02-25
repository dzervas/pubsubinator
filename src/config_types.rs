use alloc::format;
// We need to use alloc Vec, otherwise we can instatiate the KeymapConfig
// without generics
use alloc::vec::Vec;
use core::str::FromStr;
use reactor::*;

use crate::gpio::{Drive, Input, Level, Output, Pull};
use crate::keymap_mid::*;
use crate::matrix::{Matrix, MatrixDirection};

pub trait ConfigBuilder {
	type Output;
	fn build(&self) -> Self::Output;
}

#[derive(Debug, Default)]
pub struct KeymapConfig {
	pub layers: Vec<Vec<Vec<&'static str>>>,
	pub hold_cycles: u16,
}

impl ConfigBuilder for KeymapConfig {
	type Output = Keymap;
	fn build(&self) -> Self::Output {
		let layers = self
			.layers
			.iter()
			.map(|layer| {
				layer
					.iter()
					.map(|row| {
						row.iter()
							.map(|key| KeyCodeInt::Key(KeyCode::from_str(key).unwrap()))
							.collect::<Vec<KeyCodeInt>>()
					})
					.collect::<Vec<Vec<KeyCodeInt>>>()
			})
			.collect::<Vec<Vec<Vec<KeyCodeInt>>>>();

		Keymap::new(layers, self.hold_cycles)
	}
}

#[derive(Debug, Default)]
pub struct MatrixConfig {
	pub inputs: Vec<MatrixConfigInputsType>,
	pub outputs: Vec<MatrixConfigOutputsType>,
	pub direction: &'static str,
}

impl ConfigBuilder for MatrixConfig {
	type Output = Matrix<'static, Input<'static>, Output<'static>>;
	fn build(&self) -> Self::Output {
		let inputs = self.inputs.iter().map(|input| input.to_input()).collect::<Vec<Input>>();
		let outputs = self
			.outputs
			.iter()
			.map(|output| output.to_output())
			.collect::<Vec<Output>>();

		Matrix::new(inputs, outputs, MatrixDirection::from_str(self.direction).unwrap())
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MatrixConfigInputsType {
	pub pin: &'static str,
	pub pull: &'static str,
}

// TODO: Move the nrf-specific pin stuff to gpio.rs
impl MatrixConfigInputsType {
	fn to_input<'a>(self) -> Input<'a> {
		let pull = Pull::from_str(self.pull).unwrap();

		let parts = self.pin.split('.');
		let (port, pin) = if let [port_str, pin_str] = parts.into_iter().collect::<Vec<&str>>().as_slice() {
			let port = port_str
				.parse::<u8>()
				.expect(format!("Invalid port number for pin `{}`", self.pin).as_str());
			let pin = pin_str
				.parse::<u8>()
				.expect(format!("Invalid pin number for pin `{}`", self.pin).as_str());
			(port, pin)
		} else {
			panic!("Invalid pin format `{}`", self.pin)
		};
		let anypin = unsafe { embassy_nrf::gpio::AnyPin::steal(port * 32 + pin) };

		embassy_nrf::gpio::Input::new(anypin, pull.into())
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MatrixConfigOutputsType {
	pub pin: &'static str,
	pub drive: &'static str,
	pub level: &'static str,
}

impl MatrixConfigOutputsType {
	fn to_output<'a>(self) -> Output<'a> {
		let drive = Drive::from_str(self.drive).unwrap();
		let level = Level::from_str(self.level).unwrap();

		let parts = self.pin.split('.');
		let (port, pin) = if let [port_str, pin_str] = parts.into_iter().collect::<Vec<&str>>().as_slice() {
			let port = port_str
				.parse::<u8>()
				.expect(format!("Invalid port number for pin `{}`", self.pin).as_str());
			let pin = pin_str
				.parse::<u8>()
				.expect(format!("Invalid pin number for pin `{}`", self.pin).as_str());
			(port, pin)
		} else {
			panic!("Invalid pin format `{}`", self.pin)
		};
		let anypin = unsafe { embassy_nrf::gpio::AnyPin::steal(port * 32 + pin) };

		embassy_nrf::gpio::Output::new(anypin, level.into(), drive.into())
	}
}

// #[derive(Debug, Clone, Default)]
// pub struct HidConfig {
// 	pub descriptors: Vec<&'static str>,
// }

// impl ConfigBuilder for HidConfig {
// 	type Output = ();
// 	fn build(&self) -> Self::Output {
// 	}
// }
