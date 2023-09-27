pub trait KeyboardPoll {
	async fn poll(&mut self);
}

#[derive(Default)]
pub enum KeyEvent {
	Pressed,
	#[default]
	Released,
	Held,
}

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
