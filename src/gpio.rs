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
			_ => unreachable!(),
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
