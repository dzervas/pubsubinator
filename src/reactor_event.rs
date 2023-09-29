use defmt::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum KeyCode {
	N0,
	N1,
	N2,
	N3,
	N4,
	N5,
	N6,
	N7,
	N8,
	N9,

	INT1,
	INT2,
	INT3,
	INT4,
	INT5,
	INT6,
	INT7,
	INT8,
	INT9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum KeyEvent {
	Pressed(KeyCode),
	Released(KeyCode),
	Held(KeyCode),
	DoublePressed(KeyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum ReactorEvent {
	// Keyboard
	Key(KeyEvent),
	Locks { caps: bool, num: bool, scroll: bool },

	// Mouse
	// TODO: Handle the mouse wheel
	Mouse { x: u32, y: u32 },

	// Battery percentage report
	Battery(u8),

	// Simple LED control
	LED(bool),
	LEDAnalog(u8),
	RGBLED { r: u8, g: u8, b: u8 },

	// TODO: LED strip
	// TODO: Screen (widgets?)
}
