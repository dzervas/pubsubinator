extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ExprArray, Expr, parse::Parse, parse::ParseStream, Result};

struct SubscribersTaskInput {
	channel: Expr,
	subscribers: ExprArray,
}

impl Parse for SubscribersTaskInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let channel: Expr = input.parse()?;
		input.parse::<syn::token::Comma>()?;
		let subscribers: ExprArray = input.parse()?;

		Ok(Self {
			channel,
			subscribers,
		})
	}
}

#[proc_macro]
pub fn subscribers_task(input: TokenStream) -> TokenStream {
	let inputs = syn::parse_macro_input!(input as SubscribersTaskInput);

	let channel = inputs.channel;
	let subscribers = inputs.subscribers;

	let subscribers_count = subscribers.elems.len();

	let expanded = quote! {
		{
			#[embassy_executor::task]
			async fn subscriber_task(mut subscribers: [&'static mut dyn reactor::RSubscriber; #subscribers_count]) {
				// Expects subscriber to be a global but that's fine?
				let mut listener: embassy_sync::pubsub::Subscriber<_, _, _, _, _> = #channel.subscriber().unwrap();
				info!("Subscriber task started");

				loop {
					let msg = listener.next_message_pure().await;

					info!("[subscriber] Got a message: {:?}", msg);

					// TODO: Turn this into a join of all pollers
					for sub in &mut subscribers {
						if sub.is_supported(msg.clone()) {
							sub.push(msg.clone()).await;
						}
					}
				}
			}

			subscriber_task(#subscribers)
		}
	};

	TokenStream::from(expanded)
}
