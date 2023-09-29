use core::pin::Pin;

use alloc::{vec::Vec, boxed::Box};
use futures::Future;

use crate::reactor_event::*;

pub trait Producer {
	// async fn setup(&mut self);
	// TODO: Support partial state
	// async fn get_state(&self) -> ReactorEvent;

	fn setup(&mut self) -> Pin<Box<dyn Future<Output = ()>>>;
	fn get_state(&mut self) -> Pin<Box<dyn Future<Output = Option<ReactorEvent>> + '_>>;
}

pub trait Interrupted: Producer {
	async fn handler(&mut self);
}

pub trait Polled: Producer {
	async fn poll(&mut self);
}

impl<T: Polled> Interrupted for T {
	async fn handler(&mut self) {
		self.poll().await;
	}
}

pub trait Consumer {
	fn setup() -> Self where Self: Sized;
	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

pub struct Reactor {
	// TODO: We need a builder?
	pub producers: Vec<Box<dyn Producer>>,
	pub consumers : Vec<Box<dyn Consumer>>,
}

impl Reactor {
	pub async fn setup(&mut self) {
		for p in self.producers.iter_mut() {
			p.setup().await;
		}

		for _c in self.consumers.iter_mut() {
			// TODO: Pass the objects in this function and return an object
			// c.setup().await;
		}
	}

	pub async fn react(&mut self) {
		for p in self.producers.iter_mut() {
			for c in self.consumers.iter_mut() {
				match p.get_state().await {
					Some(event) => c.push(event).await,
					None => {},
				}
			}
		}
	}
}
