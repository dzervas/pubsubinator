use core::ops::{Deref, DerefMut};

use strum::EnumString;

#[cfg(feature = "nrf")]
pub type Input<'a> = embassy_nrf::gpio::Input<'a, embassy_nrf::gpio::AnyPin>;
#[cfg(feature = "nrf")]
pub type Output<'a> = embassy_nrf::gpio::Output<'a, embassy_nrf::gpio::AnyPin>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
pub enum Pull {
	None,
	Up,
	Down,
}

impl Default for Pull {
	fn default() -> Self {
		Self::None
	}
}

#[cfg(feature = "nrf")]
impl From<embassy_nrf::gpio::Pull> for Pull {
	fn from(pull: embassy_nrf::gpio::Pull) -> Self {
		match pull {
			embassy_nrf::gpio::Pull::None => Self::None,
			embassy_nrf::gpio::Pull::Up => Self::Up,
			embassy_nrf::gpio::Pull::Down => Self::Down,
		}
	}
}

#[cfg(feature = "nrf")]
impl From<Pull> for embassy_nrf::gpio::Pull {
	fn from(pull: Pull) -> embassy_nrf::gpio::Pull {
		match pull {
			Pull::None => embassy_nrf::gpio::Pull::None,
			Pull::Up => embassy_nrf::gpio::Pull::Up,
			Pull::Down => embassy_nrf::gpio::Pull::Down,
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
pub enum Level {
	Low,
	High,
}

impl Default for Level {
	fn default() -> Self {
		Self::Low
	}
}

#[cfg(feature = "nrf")]
impl From<embassy_nrf::gpio::Level> for Level {
	fn from(level: embassy_nrf::gpio::Level) -> Self {
		match level {
			embassy_nrf::gpio::Level::Low => Self::Low,
			embassy_nrf::gpio::Level::High => Self::High,
		}
	}
}

#[cfg(feature = "nrf")]
impl From<Level> for embassy_nrf::gpio::Level {
	fn from(level: Level) -> embassy_nrf::gpio::Level {
		match level {
			Level::Low => embassy_nrf::gpio::Level::Low,
			Level::High => embassy_nrf::gpio::Level::High,
		}
	}
}

impl From<bool> for Level {
	fn from(val: bool) -> Self {
		match val {
			true => Self::High,
			false => Self::Low,
		}
	}
}

impl From<Level> for bool {
	fn from(level: Level) -> bool {
		match level {
			Level::Low => false,
			Level::High => true,
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
pub enum Drive {
	Standard,
	High,
}

impl Default for Drive {
	fn default() -> Self {
		Self::Standard
	}
}

#[cfg(feature = "nrf")]
impl From<embassy_nrf::gpio::OutputDrive> for Drive {
	fn from(drive: embassy_nrf::gpio::OutputDrive) -> Self {
		match drive {
			embassy_nrf::gpio::OutputDrive::Standard => Self::Standard,
			embassy_nrf::gpio::OutputDrive::HighDrive => Self::High,
		}
	}
}

#[cfg(feature = "nrf")]
impl From<Drive> for embassy_nrf::gpio::OutputDrive {
	fn from(drive: Drive) -> embassy_nrf::gpio::OutputDrive {
		match drive {
			Drive::Standard => embassy_nrf::gpio::OutputDrive::Standard,
			Drive::High => embassy_nrf::gpio::OutputDrive::HighDrive,
		}
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
pub enum Pin {
	P0_00,
	P0_01,
	P0_02,
	P0_03,
	P0_04,
	P0_05,
	P0_06,
	P0_07,
	P0_08,
	P0_09,
	P0_10,
	P0_11,
	P0_12,
	P0_13,
	P0_14,
	P0_15,
	P0_16,
	P0_17,
	P0_18,
	P0_19,
	P0_20,
	P0_21,
	P0_22,
	P0_23,
	P0_24,
	P0_25,
	P0_26,
	P0_27,
	P0_28,
	P0_29,
	P0_30,
	P0_31,
	P1_00,
	P1_01,
	P1_02,
	P1_03,
	P1_04,
	P1_05,
	P1_06,
	P1_07,
	P1_08,
	P1_09,
	P1_10,
	P1_11,
	P1_12,
	P1_13,
	P1_14,
	P1_15,
	P1_16,
	P1_17,
	P1_18,
	P1_19,
	P1_20,
	P1_21,
	P1_22,
	P1_23,
	P1_24,
	P1_25,
	P1_26,
	P1_27,
	P1_28,
	P1_29,
	P1_30,
	P1_31,
}

// TODO: There's no way this can't be done with a macro or something
// TODO: Define board specific aliases (arduino, feather, etc)
// #[cfg(feature = "nrf")]
// impl From<embassy_nrf::gpio::AnyPin> for Pin {
// 	fn from(pin: embassy_nrf::gpio::AnyPin) -> Self {
// 		match pin {
// 			embassy_nrf::gpio::AnyPin::P0_00 => Self::P0_00,
// 			embassy_nrf::gpio::AnyPin::P0_01 => Self::P0_01,
// 			embassy_nrf::gpio::AnyPin::P0_02 => Self::P0_02,
// 			embassy_nrf::gpio::AnyPin::P0_03 => Self::P0_03,
// 			embassy_nrf::gpio::AnyPin::P0_04 => Self::P0_04,
// 			embassy_nrf::gpio::AnyPin::P0_05 => Self::P0_05,
// 			embassy_nrf::gpio::AnyPin::P0_06 => Self::P0_06,
// 			embassy_nrf::gpio::AnyPin::P0_07 => Self::P0_07,
// 			embassy_nrf::gpio::AnyPin::P0_08 => Self::P0_08,
// 			embassy_nrf::gpio::AnyPin::P0_09 => Self::P0_09,
// 			embassy_nrf::gpio::AnyPin::P0_10 => Self::P0_10,
// 			embassy_nrf::gpio::AnyPin::P0_11 => Self::P0_11,
// 			embassy_nrf::gpio::AnyPin::P0_12 => Self::P0_12,
// 			embassy_nrf::gpio::AnyPin::P0_13 => Self::P0_13,
// 			embassy_nrf::gpio::AnyPin::P0_14 => Self::P0_14,
// 			embassy_nrf::gpio::AnyPin::P0_15 => Self::P0_15,
// 			embassy_nrf::gpio::AnyPin::P0_16 => Self::P0_16,
// 			embassy_nrf::gpio::AnyPin::P0_17 => Self::P0_17,
// 			embassy_nrf::gpio::AnyPin::P0_18 => Self::P0_18,
// 			embassy_nrf::gpio::AnyPin::P0_19 => Self::P0_19,
// 			embassy_nrf::gpio::AnyPin::P0_20 => Self::P0_20,
// 			embassy_nrf::gpio::AnyPin::P0_21 => Self::P0_21,
// 			embassy_nrf::gpio::AnyPin::P0_22 => Self::P0_22,
// 			embassy_nrf::gpio::AnyPin::P0_23 => Self::P0_23,
// 			embassy_nrf::gpio::AnyPin::P0_24 => Self::P0_24,
// 			embassy_nrf::gpio::AnyPin::P0_25 => Self::P0_25,
// 			embassy_nrf::gpio::AnyPin::P0_26 => Self::P0_26,
// 			embassy_nrf::gpio::AnyPin::P0_27 => Self::P0_27,
// 			embassy_nrf::gpio::AnyPin::P0_28 => Self::P0_28,
// 			embassy_nrf::gpio::AnyPin::P0_29 => Self::P0_29,
// 			embassy_nrf::gpio::AnyPin::P0_30 => Self::P0_30,
// 			embassy_nrf::gpio::AnyPin::P0_31 => Self::P0_31,
// 			embassy_nrf::gpio::AnyPin::P1_00 => Self::P1_00,
// 			embassy_nrf::gpio::AnyPin::P1_01 => Self::P1_01,
// 			embassy_nrf::gpio::AnyPin::P1_02 => Self::P1_02,
// 			embassy_nrf::gpio::AnyPin::P1_03 => Self::P1_03,
// 			embassy_nrf::gpio::AnyPin::P1_04 => Self::P1_04,
// 			embassy_nrf::gpio::AnyPin::P1_05 => Self::P1_05,
// 			embassy_nrf::gpio::AnyPin::P1_06 => Self::P1_06,
// 			embassy_nrf::gpio::AnyPin::P1_07 => Self::P1_07,
// 			embassy_nrf::gpio::AnyPin::P1_08 => Self::P1_08,
// 			embassy_nrf::gpio::AnyPin::P1_09 => Self::P1_09,
// 			embassy_nrf::gpio::AnyPin::P1_10 => Self::P1_10,
// 			embassy_nrf::gpio::AnyPin::P1_11 => Self::P1_11,
// 			embassy_nrf::gpio::AnyPin::P1_12 => Self::P1_12,
// 			embassy_nrf::gpio::AnyPin::P1_13 => Self::P1_13,
// 			embassy_nrf::gpio::AnyPin::P1_14 => Self::P1_14,
// 			embassy_nrf::gpio::AnyPin::P1_15 => Self::P1_15,
// 			embassy_nrf::gpio::AnyPin::P1_16 => Self::P1_16,
// 			embassy_nrf::gpio::AnyPin::P1_17 => Self::P1_17,
// 			embassy_nrf::gpio::AnyPin::P1_18 => Self::P1_18,
// 			embassy_nrf::gpio::AnyPin::P1_19 => Self::P1_19,
// 			embassy_nrf::gpio::AnyPin::P1_20 => Self::P1_20,
// 			embassy_nrf::gpio::AnyPin::P1_21 => Self::P1_21,
// 			embassy_nrf::gpio::AnyPin::P1_22 => Self::P1_22,
// 			embassy_nrf::gpio::AnyPin::P1_23 => Self::P1_23,
// 			embassy_nrf::gpio::AnyPin::P1_24 => Self::P1_24,
// 			embassy_nrf::gpio::AnyPin::P1_25 => Self::P1_25,
// 			embassy_nrf::gpio::AnyPin::P1_26 => Self::P1_26,
// 			embassy_nrf::gpio::AnyPin::P1_27 => Self::P1_27,
// 			embassy_nrf::gpio::AnyPin::P1_28 => Self::P1_28,
// 			embassy_nrf::gpio::AnyPin::P1_29 => Self::P1_29,
// 			embassy_nrf::gpio::AnyPin::P1_30 => Self::P1_30,
// 			embassy_nrf::gpio::AnyPin::P1_31 => Self::P1_31,
// 			_ => panic!("Invalid pin"),
// 		}
// 	}
// }

// #[cfg(feature = "nrf")]
// impl Into<embassy_nrf::gpio::AnyPin> for Pin {
// 	fn into(self) -> embassy_nrf::gpio::AnyPin {
// 		match self {
// 			Self::P0_00 => embassy_nrf::gpio::AnyPin::P0_00,
// 			Self::P0_01 => embassy_nrf::gpio::AnyPin::P0_01,
// 			Self::P0_02 => embassy_nrf::gpio::AnyPin::P0_02,
// 			Self::P0_03 => embassy_nrf::gpio::AnyPin::P0_03,
// 			Self::P0_04 => embassy_nrf::gpio::AnyPin::P0_04,
// 			Self::P0_05 => embassy_nrf::gpio::AnyPin::P0_05,
// 			Self::P0_06 => embassy_nrf::gpio::AnyPin::P0_06,
// 			Self::P0_07 => embassy_nrf::gpio::AnyPin::P0_07,
// 			Self::P0_08 => embassy_nrf::gpio::AnyPin::P0_08,
// 			Self::P0_09 => embassy_nrf::gpio::AnyPin::P0_09,
// 			Self::P0_10 => embassy_nrf::gpio::AnyPin::P0_10,
// 			Self::P0_11 => embassy_nrf::gpio::AnyPin::P0_11,
// 			Self::P0_12 => embassy_nrf::gpio::AnyPin::P0_12,
// 			Self::P0_13 => embassy_nrf::gpio::AnyPin::P0_13,
// 			Self::P0_14 => embassy_nrf::gpio::AnyPin::P0_14,
// 			Self::P0_15 => embassy_nrf::gpio::AnyPin::P0_15,
// 			Self::P0_16 => embassy_nrf::gpio::AnyPin::P0_16,
// 			Self::P0_17 => embassy_nrf::gpio::AnyPin::P0_17,
// 			Self::P0_18 => embassy_nrf::gpio::AnyPin::P0_18,
// 			Self::P0_19 => embassy_nrf::gpio::AnyPin::P0_19,
// 			Self::P0_20 => embassy_nrf::gpio::AnyPin::P0_20,
// 			Self::P0_21 => embassy_nrf::gpio::AnyPin::P0_21,
// 			Self::P0_22 => embassy_nrf::gpio::AnyPin::P0_22,
// 			Self::P0_23 => embassy_nrf::gpio::AnyPin::P0_23,
// 			Self::P0_24 => embassy_nrf::gpio::AnyPin::P0_24,
// 			Self::P0_25 => embassy_nrf::gpio::AnyPin::P0_25,
// 			Self::P0_26 => embassy_nrf::gpio::AnyPin::P0_26,
// 			Self::P0_27 => embassy_nrf::gpio::AnyPin::P0_27,
// 			Self::P0_28 => embassy_nrf::gpio::AnyPin::P0_28,
// 			Self::P0_29 => embassy_nrf::gpio::AnyPin::P0_29,
// 			Self::P0_30 => embassy_nrf::gpio::AnyPin::P0_30,
// 			Self::P0_31 => embassy_nrf::gpio::AnyPin::P0_31,
// 			Self::P1_00 => embassy_nrf::gpio::AnyPin::P1_00,
// 			Self::P1_01 => embassy_nrf::gpio::AnyPin::P1_01,
// 			Self::P1_02 => embassy_nrf::gpio::AnyPin::P1_02,
// 			Self::P1_03 => embassy_nrf::gpio::AnyPin::P1_03,
// 			Self::P1_04 => embassy_nrf::gpio::AnyPin::P1_04,
// 			Self::P1_05 => embassy_nrf::gpio::AnyPin::P1_05,
// 			Self::P1_06 => embassy_nrf::gpio::AnyPin::P1_06,
// 			Self::P1_07 => embassy_nrf::gpio::AnyPin::P1_07,
// 			Self::P1_08 => embassy_nrf::gpio::AnyPin::P1_08,
// 			Self::P1_09 => embassy_nrf::gpio::AnyPin::P1_09,
// 			Self::P1_10 => embassy_nrf::gpio::AnyPin::P1_10,
// 			Self::P1_11 => embassy_nrf::gpio::AnyPin::P1_11,
// 			Self::P1_12 => embassy_nrf::gpio::AnyPin::P1_12,
// 			Self::P1_13 => embassy_nrf::gpio::AnyPin::P1_13,
// 			Self::P1_14 => embassy_nrf::gpio::AnyPin::P1_14,
// 			Self::P1_15 => embassy_nrf::gpio::AnyPin::P1_15,
// 			Self::P1_16 => embassy_nrf::gpio::AnyPin::P1_16,
// 			Self::P1_17 => embassy_nrf::gpio::AnyPin::P1_17,
// 			Self::P1_18 => embassy_nrf::gpio::AnyPin::P1_18,
// 			Self::P1_19 => embassy_nrf::gpio::AnyPin::P1_19,
// 			Self::P1_20 => embassy_nrf::gpio::AnyPin::P1_20,
// 			Self::P1_21 => embassy_nrf::gpio::AnyPin::P1_21,
// 			Self::P1_22 => embassy_nrf::gpio::AnyPin::P1_22,
// 			Self::P1_23 => embassy_nrf::gpio::AnyPin::P1_23,
// 			Self::P1_24 => embassy_nrf::gpio::AnyPin::P1_24,
// 			Self::P1_25 => embassy_nrf::gpio::AnyPin::P1_25,
// 			Self::P1_26 => embassy_nrf::gpio::AnyPin::P1_26,
// 			Self::P1_27 => embassy_nrf::gpio::AnyPin::P1_27,
// 			Self::P1_28 => embassy_nrf::gpio::AnyPin::P1_28,
// 			Self::P1_29 => embassy_nrf::gpio::AnyPin::P1_29,
// 			Self::P1_30 => embassy_nrf::gpio::AnyPin::P1_30,
// 			Self::P1_31 => embassy_nrf::gpio::AnyPin::P1_31,
// 			_ => panic!("Invalid pin"),
// 		}
// 	}
// }
