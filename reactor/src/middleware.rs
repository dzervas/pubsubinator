use core::pin::Pin;

use alloc::boxed::Box;
use futures::Future;

use crate::reactor_event::ReactorEvent;
use crate::RSubscriber;

pub trait Middleware {
	fn process(&mut self, value: ReactorEvent) -> Pin<Box<dyn Future<Output = Option<ReactorEvent>> + '_>>;
}

impl<T: Middleware> RSubscriber for T {
	fn push(&mut self, value: ReactorEvent) -> Pin<Box<dyn futures::Future<Output = ()> + '_>> {
		Box::pin(async move {
			self.process(value).await;
		})
	}
}
