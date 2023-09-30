use core::pin::Pin;

use alloc::boxed::Box;
use futures::Future;
#[allow(unused_imports)]
use defmt::*;

use crate::reactor_event::*;

pub trait Producer {
	// async fn setup(&mut self);

	fn setup(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

pub trait Interrupted: Producer {
	async fn handler(&mut self);
}

pub trait Polled: Producer {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: Polled> Interrupted for T {
	async fn handler(&mut self) {
		self.poll().await;
	}
}

pub trait Consumer {
	// fn setup() -> Self where Self: Sized;
	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}
