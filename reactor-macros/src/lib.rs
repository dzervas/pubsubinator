extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
// use syn::parse::{Parse, ParseStream};
// use syn::{Ident, Token, Type, Result};
// use syn::punctuated::Punctuated;

#[proc_macro]
pub fn subscribers_task(subscribers: TokenStream) -> TokenStream {
	let subscribers = syn::parse_macro_input!(subscribers as syn::ExprArray);
	let subscribers_count = subscribers.elems.len();
	// let fn_suffix = &subscribers.sig.ident;

	let expanded = quote! {
		{
			#[embassy_executor::task]
			async fn subscriber_task(mut subscribers: [&'static mut dyn RSubscriber; #subscribers_count]) {
				info!("Subscriber task started");
				// let mut subscribers = [#subscribers];
				let mut listener = crate::CHANNEL.subscriber().unwrap();
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
