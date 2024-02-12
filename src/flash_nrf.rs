use core::convert::Infallible;

use defmt::*;
use ekv::config;
use ekv::flash::PageID;
use embassy_nrf::{peripherals, qspi};

// Workaround for alignment requirements.
#[repr(C, align(4))]
struct AlignedBuf([u8; 2048]);

static mut BUF: AlignedBuf = AlignedBuf([0; 2048]);

pub struct Flash<'a> {
	qspi: qspi::Qspi<'a, peripherals::QSPI>,
}

impl<'a> ekv::flash::Flash for Flash<'a> {
	type Error = Infallible;

	fn page_count(&self) -> usize {
		config::MAX_PAGE_COUNT
	}

	async fn erase(&mut self, page_id: PageID) -> Result<(), Self::Error> {
		self.qspi
			.erase((page_id.index() * config::PAGE_SIZE) as u32)
			.await
			.unwrap();
		Ok(())
	}

	async fn read(&mut self, page_id: PageID, offset: usize, data: &mut [u8]) -> Result<(), Self::Error> {
		let address = page_id.index() * config::PAGE_SIZE + offset;
		unsafe {
			self.qspi.read(address as u32, &mut BUF.0[..data.len()]).await.unwrap();
			data.copy_from_slice(&BUF.0[..data.len()])
		}
		Ok(())
	}

	async fn write(&mut self, page_id: PageID, offset: usize, data: &[u8]) -> Result<(), Self::Error> {
		let address = page_id.index() * config::PAGE_SIZE + offset;
		unsafe {
			BUF.0[..data.len()].copy_from_slice(data);
			self.qspi.write(address as u32, &BUF.0[..data.len()]).await.unwrap();
		}
		Ok(())
	}
}

impl<'a> Flash<'a> {
	pub async fn new() -> Self {
		let mut config = qspi::Config::default();

		config.read_opcode = qspi::ReadOpcode::READ4IO;
		config.write_opcode = qspi::WriteOpcode::PP4IO;
		config.write_page_size = qspi::WritePageSize::_256BYTES;
		config.frequency = qspi::Frequency::M32;
		config.capacity = 4 * 1024 * 1024; // 4MB
		config.deep_power_down = Some(qspi::DeepPowerDownConfig {
			enter_time: 3, // tDP = 30uS
			exit_time: 3,  // tRDP = 35uS
		});

		// TODO: Configure the QSPI pins per device
		// Valid for nrf52840 DK and Particle Xenon
		let qspi = unsafe { peripherals::QSPI::steal() };
		let sck = unsafe { peripherals::P0_19::steal() };
		let csn = unsafe { peripherals::P0_17::steal() };
		let io0 = unsafe { peripherals::P0_20::steal() };
		let io1 = unsafe { peripherals::P0_21::steal() };
		let io2 = unsafe { peripherals::P0_22::steal() };
		let io3 = unsafe { peripherals::P0_23::steal() };

		let mut q: qspi::Qspi<_> = qspi::Qspi::new(qspi, crate::Irqs, sck, csn, io0, io1, io2, io3, config);

		let mut id = [1; 3];
		q.custom_instruction(0x9F, &[], &mut id).await.unwrap();
		info!("Initialized flash with ID: {:X}", id);

		// Read status register
		let mut status = [4; 1];
		unwrap!(q.custom_instruction(0x05, &[], &mut status).await);

		info!("Flash status: {:X}", status[0]);

		if status[0] & 0x40 == 0 {
			status[0] |= 0x40;

			unwrap!(q.custom_instruction(0x01, &status, &mut []).await);

			info!("Enabled Quad SPI Flash mode");
		}

		Self { qspi: q }
	}
}
