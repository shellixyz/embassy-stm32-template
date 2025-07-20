use std::{env, fs, path::PathBuf};

use indoc::formatdoc;

fn write_version_file() {
	let git_version = git_version::git_version!(fallback = "unknown");
	let mut git_revision_bytes = [0; 10];
	let (revision, modified) = git_version.split_once('-').unwrap_or((git_version, ""));
	let git_revision_len = revision.len().min(10);
	git_revision_bytes[..git_revision_len].copy_from_slice(&revision.as_bytes()[..git_revision_len]);
	let dirty = git_version == "unknown" || !modified.is_empty();
	let product_description = format!("{{project-name}} ({git_version})");
	let pkg_version_major = env::var("CARGO_PKG_VERSION_MAJOR").unwrap().parse::<u8>().unwrap();
	let pkg_version_minor = env::var("CARGO_PKG_VERSION_MINOR").unwrap().parse::<u8>().unwrap();
	let pkg_version_patch = env::var("CARGO_PKG_VERSION_PATCH").unwrap().parse::<u8>().unwrap();
	let contents = formatdoc! {"
		{% raw %}
		pub mod version {{
			pub const GIT_REVISION: &str = \"{git_version}\";
			pub const GIT_REVISION_BYTES: [u8; 10] = {git_revision_bytes:?};
			pub const GIT_REVISION_DIRTY: bool = {dirty};
			pub const PRODUCT_DESCRIPTION: &str = \"{product_description}\";
			pub const PKG_VERSION_MAJOR: u8 = {pkg_version_major};
			pub const PKG_VERSION_MINOR: u8 = {pkg_version_minor};
			pub const PKG_VERSION_PATCH: u8 = {pkg_version_patch};
		}}
		{% endraw %}
	"};
	let file_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("version.rs");
	fs::write(file_path, contents).unwrap_or_else(|_| panic!("Could not write version file"));
}

fn main() {
	write_version_file();
	println!("cargo:rustc-link-arg-bins=--nmagic");
	println!("cargo:rustc-link-arg-bins=-Tlink.x");
	println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
