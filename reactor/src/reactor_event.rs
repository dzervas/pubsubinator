use defmt::Format;
use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format, EnumString)]
pub enum KeyEvent {
	Pressed(KeyCode),
	Released(KeyCode),
	// TODO: Configurable alternate button behavior
	// Held(KeyCode),
	// DoublePressed(KeyCode),
}

impl Default for KeyEvent {
	fn default() -> Self {
		Self::Released(KeyCode::None)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Format, EnumString)]
pub enum InternalEvent {
	None,
	LayerNext,
	LayerPrev,
	LayerChange(usize),

	BLENext,
	BLEPrev,
	BLEChange(usize),
}

impl Default for InternalEvent {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format, EnumString)]
pub enum ReactorEvent {
	// Keyboard
	Key(KeyEvent),
	Locks {
		caps: bool,
		num: bool,
		scroll: bool,
	},
	KeyboardReport {
		modifier: KeyModifiers,
		keycodes: [KeyCode; 6],
	},

	// Mouse
	// TODO: Handle the mouse wheel
	Mouse {
		x: u32,
		y: u32,
	},

	Potentiometer {
		v: i16,
	},
	Joystick {
		x: i16,
		y: i16,
	},
	FullJoystick {
		x: i16,
		y: i16,
		z: i16,
	},
	Joystick6DoF {
		x: i16,
		y: i16,
		z: i16,
		rx: i16,
		ry: i16,
		rz: i16,
	},

	// Battery percentage report
	Battery(u8),

	// Simple LED control
	LED(bool),
	LEDAnalog(u8),
	RGBLED {
		r: u8,
		g: u8,
		b: u8,
	},

	// TODO: LED strip
	// TODO: Screen (widgets?)

	// Hardware
	// TODO: Why 2 dimensions? Why not 1? Why not variable?
	HardwareMappedBool(bool, usize, usize),
	HardwareMappedU8(u8, usize, usize),
	HardwareMappedU16(u16, usize, usize),
	Analog6Axis(i16, i16, i16, i16, i16, i16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Format, EnumString)]
#[repr(u8)]
pub enum KeyCode {
	None = 0x00,
	ErrorRollOver,
	/// The POST fail error.
	PostFail,
	/// An undefined error occured.
	ErrorUndefined,
	/// `a` and `A`.
	A,
	B,
	C,
	D,
	E,
	F,
	G,
	H,
	I,
	J,
	K,
	L,
	M, // 0x10
	N,
	O,
	P,
	Q,
	R,
	S,
	T,
	U,
	V,
	W,
	X,
	Y,
	Z,
	/// `1` and `!`.
	Kb1,
	/// `2` and `@`.
	Kb2,
	/// `3` and `#`.
	Kb3, // 0x20
	/// `4` and `$`.
	Kb4,
	/// `5` and `%`.
	Kb5,
	/// `6` and `^`.
	Kb6,
	/// `7` and `&`.
	Kb7,
	/// `8` and `*`.
	Kb8,
	/// `9` and `(`.
	Kb9,
	/// `0` and `)`.
	Kb0,
	Enter,
	Escape,
	BSpace,
	Tab,
	Space,
	/// `-` and `_`.
	Minus,
	/// `=` and `+`.
	Equal,
	/// `[` and `{`.
	LBracket,
	/// `]` and `}`.
	RBracket, // 0x30
	/// `\` and `|`.
	Bslash,
	/// Non-US `#` and `~` (Typically near the Enter key).
	NonUsHash,
	/// `;` and `:`.
	SColon,
	/// `'` and `"`.
	Quote,
	// How to have ` as code?
	/// \` and `~`.
	Grave,
	/// `,` and `<`.
	Comma,
	/// `.` and `>`.
	Dot,
	/// `/` and `?`.
	Slash,
	CapsLock,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7, // 0x40
	F8,
	F9,
	F10,
	F11,
	F12,
	PScreen,
	ScrollLock,
	Pause,
	Insert,
	Home,
	PgUp,
	Delete,
	End,
	PgDown,
	Right,
	Left, // 0x50
	Down,
	Up,
	NumLock,
	/// Keypad `/`
	KpSlash,
	/// Keypad `*`
	KpAsterisk,
	/// Keypad `-`.
	KpMinus,
	/// Keypad `+`.
	KpPlus,
	/// Keypad enter.
	KpEnter,
	/// Keypad 1.
	Kp1,
	Kp2,
	Kp3,
	Kp4,
	Kp5,
	Kp6,
	Kp7,
	Kp8, // 0x60
	Kp9,
	Kp0,
	KpDot,
	/// Non-US `\` and `|` (Typically near the Left-Shift key)
	NonUsBslash,
	Application, // 0x65
	/// not a key, used for errors
	Power,
	/// Keypad `=`.
	KpEqual,
	F13,
	F14,
	F15,
	F16,
	F17,
	F18,
	F19,
	F20,
	F21, // 0x70
	F22,
	F23,
	F24,
	Execute,
	Help,
	Menu,
	Select,
	Stop,
	Again,
	Undo,
	Cut,
	Copy,
	Paste,
	Find,
	Mute,
	VolUp, // 0x80
	VolDown,
	/// Deprecated.
	LockingCapsLock,
	/// Deprecated.
	LockingNumLock,
	/// Deprecated.
	LockingScrollLock,
	/// Keypad `,`, also used for the brazilian keypad period (.) key.
	KpComma,
	/// Used on AS/400 keyboard
	KpEqualSign,
	Intl1,
	Intl2,
	Intl3,
	Intl4,
	Intl5,
	Intl6,
	Intl7,
	Intl8,
	Intl9,
	Lang1, // 0x90
	Lang2,
	Lang3,
	Lang4,
	Lang5,
	Lang6,
	Lang7,
	Lang8,
	Lang9,
	AltErase,
	SysReq,
	Cancel,
	Clear,
	Prior,
	Return,
	Separator,
	Out, // 0xA0
	Oper,
	ClearAgain,
	CrSel,
	ExSel,

	// According to QMK, 0xA5-0xDF are not usable on modern keyboards

	// Modifiers
	/// Left Control.
	LCtrl = 0xE0,
	/// Left Shift.
	LShift,
	/// Left Alt.
	LAlt,
	/// Left GUI (the Windows key).
	LGui,
	/// Right Control.
	RCtrl,
	/// Right Shift.
	RShift,
	/// Right Alt (or Alt Gr).
	RAlt,
	/// Right GUI (the Windows key).
	RGui, // 0xE7

	// Unofficial
	MediaPlayPause = 0xE8,
	MediaStopCD,
	MediaPreviousSong,
	MediaNextSong,
	MediaEjectCD,
	MediaVolUp,
	MediaVolDown,
	MediaMute,
	MediaWWW, // 0xF0
	MediaBack,
	MediaForward,
	MediaStop,
	MediaFind,
	MediaScrollUp,
	MediaScrollDown,
	MediaEdit,
	MediaSleep,
	MediaCoffee,
	MediaRefresh,
	MediaCalc, // 0xFB
}

impl Default for KeyCode {
	fn default() -> Self {
		Self::None
	}
}

impl Into<u8> for KeyCode {
	fn into(self) -> u8 {
		self as u8
	}
}

impl From<u8> for KeyCode {
	fn from(value: u8) -> Self {
		if value > 0xFB {
			Self::None
		} else {
			unsafe { core::mem::transmute(value) }
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format, EnumString)]
pub enum KeyCodeInt {
	None,
	Key(KeyCode),
	Internal(InternalEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct KeyModifiers {
	pub lctrl: bool,
	pub lshift: bool,
	pub lalt: bool,
	pub lgui: bool,
	pub rctrl: bool,
	pub rshift: bool,
	pub ralt: bool,
	pub rgui: bool,
}

impl Default for KeyModifiers {
	fn default() -> Self {
		Self {
			lctrl: false,
			lshift: false,
			lalt: false,
			lgui: false,
			rctrl: false,
			rshift: false,
			ralt: false,
			rgui: false,
		}
	}
}

impl Into<u8> for KeyModifiers {
	fn into(self) -> u8 {
		(self.lctrl as u8) << 0
			| (self.lshift as u8) << 1
			| (self.lalt as u8) << 2
			| (self.lgui as u8) << 3
			| (self.rctrl as u8) << 4
			| (self.rshift as u8) << 5
			| (self.ralt as u8) << 6
			| (self.rgui as u8) << 7
	}
}

impl From<u8> for KeyModifiers {
	fn from(value: u8) -> Self {
		Self {
			lctrl: value & 1 << 0 != 0,
			lshift: value & 1 << 1 != 0,
			lalt: value & 1 << 2 != 0,
			lgui: value & 1 << 3 != 0,
			rctrl: value & 1 << 4 != 0,
			rshift: value & 1 << 5 != 0,
			ralt: value & 1 << 6 != 0,
			rgui: value & 1 << 7 != 0,
		}
	}
}
