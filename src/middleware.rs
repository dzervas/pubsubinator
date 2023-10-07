use crate::reactor_event::ReactorEvent;

pub trait PublisherMiddleware<T> {
	async fn process(&mut self, value: T) -> Option<ReactorEvent>;
}

pub trait SubscriberMiddleware<T> {
	async fn process(&mut self, value: ReactorEvent) -> Option<T>;
}
