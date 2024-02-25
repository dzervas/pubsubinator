use crate::constants;

pub struct Report {
	pub collections: &'static [Collection],
}

pub struct Collection {
	pub collection_type: constants::CollectionID,
	pub usage_page: constants::UsagePageID,
	pub usages: &'static [Usage],
}

pub struct Usage {
	pub usage_page: constants::UsagePageID,
	pub min: Option<u16>,
	pub max: Option<u16>,
	pub packed_bits: Option<u8>,
	pub items: &'static [Item],
}

pub enum ItemSettings {
	Data,
	Variable,
	Absolute,
	Relative,
	Array,
	Constant,
}

pub struct Item {
	pub settings: ItemSettings,
	pub value: u16,
}
