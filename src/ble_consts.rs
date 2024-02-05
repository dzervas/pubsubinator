pub enum HidReportType {
	// Main items
	HidInput      = 0x80,
	HidOutput     = 0x90,
	Feature       = 0xb0,
	Collection    = 0xa0,
	EndCollection = 0xc0,


	// Global items
	UsagePage       = 0x04,
	LogicalMinimum  = 0x14,
	LogicalMaximum  = 0x24,
	PhysicalMinimum = 0x34,
	PhysicalMaximum = 0x44,
	UnitExponent    = 0x54,
	Unit            = 0x64,
	ReportSize      = 0x74, //bits
	ReportID        = 0x84,
	ReportCount     = 0x94, //bytes
	Push            = 0xa4,
	Pop             = 0xb4,

	// Local items
	Usage             = 0x08,
	UsageMinimum      = 0x18,
	UsageMaximum      = 0x28,
	DesignatorIndex   = 0x38,
	DesignatorMinimum = 0x48,
	DesignatorMaximum = 0x58,
	StringIndex       = 0x78,
	StringMinimum     = 0x88,
	StringMaximum     = 0x98,
	Delimiter         = 0xa8,
}

pub enum HidReportMapType {
	// Main items
	HidInput(u8),
	HidOutput(u8),
	Feature(u8),
	Collection(u8),
	EndCollection,


	// Global items
	UsagePage(u8),
	LogicalMinimum(u8),
	LogicalMaximum(u8),
	PhysicalMinimum(u8),
	PhysicalMaximum(u8),
	UnitExponent(u8),
	Unit(u8),
	ReportSize(u8), //bits
	ReportID(u8),
	ReportCount(u8), //bytes
	Push(u8),
	Pop(u8),

	// Local items
	Usage(u8),
	UsageW(u8, u8),
	UsageMinimum(u8),
	UsageMaximum(u8),
	DesignatorIndex(u8),
	DesignatorMinimum(u8),
	DesignatorMaximum(u8),
	StringIndex(u8),
	StringMinimum(u8),
	StringMaximum(u8),
	Delimiter(u8),
}

impl HidReportMapType {
	pub fn to_bytes(&self) -> ([u8; 3], usize) {
		match self {
			HidReportMapType::HidInput(report_id)                   => ([1 + HidReportType::HidInput as u8, *report_id, 0], 2),
			HidReportMapType::HidOutput(report_id)                  => ([1 + HidReportType::HidOutput as u8, *report_id, 0], 2),
			HidReportMapType::Feature(report_id)                    => ([1 + HidReportType::Feature as u8, *report_id, 0], 2),
			HidReportMapType::Collection(collection_type)           => ([1 + HidReportType::Collection as u8, *collection_type, 0], 2),
			HidReportMapType::EndCollection                         => ([HidReportType::EndCollection as u8, 0, 0], 2),
			HidReportMapType::UsagePage(usage_page)                 => ([1 + HidReportType::UsagePage as u8, *usage_page, 0], 2),
			HidReportMapType::LogicalMinimum(logical_minimum)       => ([1 + HidReportType::LogicalMinimum as u8, *logical_minimum, 0], 2),
			HidReportMapType::LogicalMaximum(logical_maximum)       => ([1 + HidReportType::LogicalMaximum as u8, *logical_maximum, 0], 2),
			HidReportMapType::PhysicalMinimum(physical_minimum)     => ([1 + HidReportType::PhysicalMinimum as u8, *physical_minimum, 0], 2),
			HidReportMapType::PhysicalMaximum(physical_maximum)     => ([1 + HidReportType::PhysicalMaximum as u8, *physical_maximum, 0], 2),
			HidReportMapType::UnitExponent(unit_exponent)           => ([1 + HidReportType::UnitExponent as u8, *unit_exponent, 0], 2),
			HidReportMapType::Unit(unit)                            => ([1 + HidReportType::Unit as u8, *unit, 0], 2),
			HidReportMapType::ReportSize(report_size)               => ([1 + HidReportType::ReportSize as u8, *report_size, 0], 2),
			HidReportMapType::ReportID(report_id)                   => ([1 + HidReportType::ReportID as u8, *report_id, 0], 2),
			HidReportMapType::ReportCount(report_count)             => ([1 + HidReportType::ReportCount as u8, *report_count, 0], 2),
			HidReportMapType::Push(push)                            => ([1 + HidReportType::Push as u8, *push, 0], 2),
			HidReportMapType::Pop(pop)                              => ([1 + HidReportType::Pop as u8, *pop, 0], 2),
			HidReportMapType::Usage(usage)                          => ([1 + HidReportType::Usage as u8, *usage, 0], 2),
			HidReportMapType::UsageW(usage_page, usage)             => ([2 + HidReportType::Usage as u8, *usage_page, *usage], 3),
			HidReportMapType::UsageMinimum(usage_minimum)           => ([1 + HidReportType::UsageMinimum as u8, *usage_minimum, 0], 2),
			HidReportMapType::UsageMaximum(usage_maximum)           => ([1 + HidReportType::UsageMaximum as u8, *usage_maximum, 0], 2),
			HidReportMapType::DesignatorIndex(designator_index)     => ([1 + HidReportType::DesignatorIndex as u8, *designator_index, 0], 2),
			HidReportMapType::DesignatorMinimum(designator_minimum) => ([1 + HidReportType::DesignatorMinimum as u8, *designator_minimum, 0], 2),
			HidReportMapType::DesignatorMaximum(designator_maximum) => ([1 + HidReportType::DesignatorMaximum as u8, *designator_maximum, 0], 2),
			HidReportMapType::StringIndex(string_index)             => ([1 + HidReportType::StringIndex as u8, *string_index, 0], 2),
			HidReportMapType::StringMinimum(string_minimum)         => ([1 + HidReportType::StringMinimum as u8, *string_minimum, 0], 2),
			HidReportMapType::StringMaximum(string_maximum)         => ([1 + HidReportType::StringMaximum as u8, *string_maximum, 0], 2),
			HidReportMapType::Delimiter(delimiter)                  => ([1 + HidReportType::Delimiter as u8, *delimiter, 0], 2),
		}
	}
}
