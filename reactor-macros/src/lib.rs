extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprArray, Result};

struct SubscribersTaskInput {
	channel: Expr,
	subscribers: ExprArray,
	middleware: ExprArray,
}

impl Parse for SubscribersTaskInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let channel: Expr = input.parse()?;
		input.parse::<syn::token::Comma>()?;
		let subscribers: ExprArray = input.parse()?;
		input.parse::<syn::token::Comma>()?;
		let middleware: ExprArray = input.parse()?;

		Ok(Self {
			channel,
			subscribers,
			middleware,
		})
	}
}

#[proc_macro]
pub fn subscribers_task(input: TokenStream) -> TokenStream {
	let inputs = syn::parse_macro_input!(input as SubscribersTaskInput);

	let channel = inputs.channel;
	let subscribers = inputs.subscribers;
	let middleware = inputs.middleware;

	let subscribers_count = subscribers.elems.len();
	let middleware_count = middleware.elems.len();

	let expanded = quote! {
		{
			#[embassy_executor::task]
			async fn subscriber_task(mut subscribers: [&'static mut dyn reactor::RSubscriber; #subscribers_count], mut middleware: [&'static mut dyn reactor::middleware::Middleware; #middleware_count]) {
				// Expects subscriber to be a global but that's fine?
				let mut listener = #channel.subscriber().unwrap();
				let publisher = #channel.publisher().unwrap();
				info!("Subscriber task started");

				loop {
					let msg = listener.next_message_pure().await;

					info!("[subscriber] Got a message: {:?}", msg);

					for mid in &mut middleware {
						if let Some(msg) = mid.process(msg.clone()).await {
							publisher.publish(msg).await;
						}
					}

					// TODO: Turn this into a join of all pollers
					for sub in &mut subscribers {
						if sub.is_supported(msg.clone()) {
							sub.push(msg.clone()).await;
						}
					}
				}
			}

			subscriber_task(#subscribers, #middleware)
		}
	};

	TokenStream::from(expanded)
}
