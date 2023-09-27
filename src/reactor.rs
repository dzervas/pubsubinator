use alloc::{vec::Vec, boxed::Box};

use crate::reactor_event::*;

pub trait Producer {
	async fn setup(&mut self);
	// TODO: Support partial state
	async fn get_state(&self) -> ReactorEvent;
}

pub trait Interrupted: Producer {
	async fn handler(&mut self);
}

pub trait Polled: Producer {
	async fn poll(&mut self);
}

// impl<T: Polled> Interrupted for T {
// 	async fn handler(&mut self) {
// 		self.poll().await;
// 	}
// }

pub trait Consumer {
	async fn setup(&mut self);
	async fn push(&mut self, value: ReactorEvent);
}

pub struct Reactor<P: Producer, C: Consumer> {
	// TODO: We need a builder?
	pub producers: Vec<Box<P>>,
	pub consumers : Vec<Box<C>>,
}

impl<P: Producer, C: Consumer> Reactor<P, C> {
	pub async fn setup(&mut self) {
		for p in self.producers.iter_mut() {
			p.setup().await;
		}

		for c in self.consumers.iter_mut() {
			c.setup().await;
		}
	}

	pub async fn react(&mut self) {
		for p in self.producers.iter() {
			for c in self.consumers.iter_mut() {
				c.push(p.get_state().await).await;
			}
		}
	}
}
