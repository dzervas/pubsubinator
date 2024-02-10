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

macro_rules! generate_pin_enum {
	($base:path, $($name:ident),* $(,)?) => {
		#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
		pub enum Pin {
			$($name,)*
		}

		#[cfg(feature = "nrf")]
		impl Pin {
			pub unsafe fn to_input(self, pull: Pull) -> Input<'static> {
				use embassy_nrf::gpio::Pin;
				match self {
					$(
						Self::$name => embassy_nrf::gpio::Input::new(embassy_nrf::peripherals::$name::steal().degrade(), pull.into()),
					)*
				}
			}

			pub unsafe fn to_output(self, drive: Drive, level: Level) -> Output<'static> {
				use embassy_nrf::gpio::Pin;
				match self {
					$(
						Self::$name => embassy_nrf::gpio::Output::new(embassy_nrf::peripherals::$name::steal().degrade(), level.into(), drive.into()),
					)*
				}
			}
		}
	};
}

// TODO: Define board specific aliases (arduino, feather, etc)
#[cfg(feature = "nrf")]
generate_pin_enum!(
	embassy_nrf::peripherals,
	P0_00,
	P0_01,
	P0_02,
	P0_03,
	P0_04,
	P0_05,
	P0_06,
	P0_07,
	P0_08,
	// P0_09,
	// P0_10,
	P0_11,
	P0_12,
	P0_13,
	P0_14,
	P0_15,
	P0_16,
	P0_17,
	// P0_18,
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
);
