use core::pin::Pin;

use alloc::boxed::Box;
use futures::Future;
#[allow(unused_imports)]
use defmt::*;

use crate::reactor_event::*;

pub trait RPublisher {
	// async fn setup(&mut self);

	fn setup(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

pub trait Interrupted: RPublisher {
	async fn handler(&mut self);
}

pub trait Polled: RPublisher {
	fn poll(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: Polled> Interrupted for T {
	async fn handler(&mut self) {
		self.poll().await;
	}
}

pub trait RSubscriber {
	// TODO: Keep the type and add an event `Any` to the enum or let the subscriber define the whole logic?
	// type SupportedEvents: IntoIterator<Item = ReactorEvent>;
	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()> + '_>>;
	fn is_supported(&self, event: ReactorEvent) -> bool {
		// Self::SupportedEvents::into_iter().any(|e| e == event)
		true
	}
}
