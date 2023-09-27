pub enum KeyEvent {
	Pressed,
	Released,
	Held,
	DoublePressed,
}

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
