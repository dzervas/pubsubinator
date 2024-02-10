//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use convert_case::{Case, Casing};
use std::fs::{copy, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};
use toml;

fn value_to_rust(section: &str, key: &str, value: &toml::Value) -> String {
	match value {
		toml::Value::String(s) => format!("\"{}\"", s),
		toml::Value::Integer(i) => i.to_string(),
		toml::Value::Float(f) => f.to_string(),
		toml::Value::Boolean(b) => b.to_string(),
		toml::Value::Array(arr) => {
			let elements = arr
				.iter()
				.map(|v| value_to_rust(section, key, v))
				.collect::<Vec<_>>()
				.join(", ");
			format!("vec![{}]", elements)
		},
		toml::Value::Table(table) => {
			let field_type = key.to_case(Case::Pascal);
			let fields = table
				.iter()
				.map(|(k, v)| format!("{}: {}", k, value_to_rust(section, key, v)))
				.collect::<Vec<_>>()
				.join(",\n\t");
			format!("{}Config{}Type {{\n\t{}\n}}", section, field_type, fields) // Assuming you have a corresponding struct
		},
		// Handle other TOML types as needed
		_ => panic!("Unsupported TOML value type"),
	}
}

fn main() {
	// Put `memory.x` in our output directory and ensure it's
	// on the linker search path.
	let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
	copy("memory.x", out.join("memory.x")).unwrap();
	println!("cargo:rustc-link-search={}", out.display());

	// By default, Cargo will re-run a build script whenever
	// any file in the project changes. By specifying `memory.x`
	// here, we ensure the build script is only re-run when
	// `memory.x` is changed.
	println!("cargo:rerun-if-changed=memory.x");

	println!("cargo:rustc-link-arg-bins=--nmagic");
	println!("cargo:rustc-link-arg-bins=-Tlink.x");
	println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

	let board_path = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("boards")
		.join("example.toml");
	let board = fs::read_to_string(&board_path)
		.expect(format!("Could not read config file {}", (&board_path).to_str().unwrap()).as_str());
	println!("cargo:rerun-if-changed={:?}", board_path);

	let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("config.rs");
	let mut config = File::create(&config_path).unwrap();
	println!("cargo:rerun-if-changed={:?}", config_path);

	writeln!(config, "use alloc::vec;").unwrap();
	writeln!(config, "use lazy_static::lazy_static;").unwrap();
	writeln!(config, "use crate::config_types::*;\n").unwrap();

	writeln!(config, "lazy_static! {{").unwrap();
	for (section, items) in toml::from_str::<toml::Table>(board.as_str()).unwrap() {
		if section == "global" {
			continue;
		}

		let name = section.to_case(Case::Upper);
		let field_type = section.to_case(Case::Pascal);
		let fields = items
			.as_table()
			.unwrap()
			.into_iter()
			.map(|(k, v)| format!("\t{}: {}", k, value_to_rust(&field_type, k, v)))
			.collect::<Vec<String>>()
			.join(",\n\t");

		writeln!(
			config,
			"\tpub static ref {0}: {1}Config = {1}Config {{\n\t{2},\n\t\t..Default::default()\n\t}};\n",
			name, field_type, fields
		)
		.unwrap();
	}
	writeln!(config, "}}").unwrap();

	config.sync_all().unwrap();
}
