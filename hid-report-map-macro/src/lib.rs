#![no_std]

pub mod constants;

#[macro_export]
macro_rules! hid {
	($name:ident = collection($collection_type:ident, $usage_page:ident, $usage:ident) { $($inner:tt)* }) => {
		// Collection {
		// 	collection_type: constants::CollectionID::$collection_type,
		// 	usage_page: constants::UsagePageID::$usage_page,
		// 	usage: Usage::$usage,
		// 	items: vec![$(hid!($inner)),*]
		// }
	};
}

hid! {
	keyboard = collection(Application, GenericDesktop, Keyboard) {}
}
