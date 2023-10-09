//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashMap;

use serde::Deserialize;
use serde_yaml::Value;

#[derive(Debug, Deserialize)]
struct Config {
	name: String,
	features: Vec<String>,
	structs: Vec<StructConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct StructConfig {
	#[serde(rename = "type")]
	type_name: String,

	#[serde(flatten)]
	arguments: HashMap<String, Value>,
}

fn main() {
	let config: Config = serde_yaml::from_str(include_str!(env!("TARGET_CONFIG"))).unwrap();

	println!("cargo:warning=Config: {:?}", config);

	env::set_var("CONFIG_NAME", config.name);

	for struct_config in config.structs {

	}

	// Put `memory.x` in our output directory and ensure it's
	// on the linker search path.
	let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
	File::create(out.join("memory.x"))
		.unwrap()
		.write_all(include_bytes!("memory.x"))
// 		.write_all(format!("
// MEMORY
// {{
//   FLASH    : ORIGIN = 0x{:?}, LENGTH = 0x{}
//   RAM (rw) : ORIGIN = 0x{}, LENGTH = 0x{}
// }}
// ",
// ram.starting_address, ram.size.unwrap(),
// flash.starting_address, flash.size()))
		.unwrap();
	println!("cargo:rustc-link-search={}", out.display());

	// By default, Cargo will re-run a build script whenever
	// any file in the project changes. By specifying `memory.x`
	// here, we ensure the build script is only re-run when
	// `memory.x` is changed.
	println!("cargo:rerun-if-changed=memory.x");

	println!("cargo:rustc-link-arg-bins=--nmagic");
	println!("cargo:rustc-link-arg-bins=-Tlink.x");
	println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
