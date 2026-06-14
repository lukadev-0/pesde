use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::Digest;

fn main() {
	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
	let assets_dir = Path::new("src/frontend/assets");

	println!("cargo:rerun-if-changed=migrations");
	println!("cargo:rerun-if-env-changed=PESDE_REGISTRY_EMBED_ASSETS");
	println!("cargo:rerun-if-changed={}", assets_dir.display());

	let embed_assets = env::var("PESDE_REGISTRY_EMBED_ASSETS").is_ok()
		|| env::var("PROFILE").unwrap() == "release";

	let mut assets = Assets::default();

	assets.emit("NUNITO_SANS", "fonts/nunito-sans-latin-wght-normal.woff2");

	assets.emit(
		"CSS",
		if embed_assets {
			let fs = lightningcss::bundler::FileProvider::new();
			let mut bundler = lightningcss::bundler::Bundler::new(
				&fs,
				None,
				lightningcss::stylesheet::ParserOptions::default(),
			);
			let stylesheet = bundler.bundle(&assets_dir.join("css/main.css")).unwrap();
			let css = stylesheet
				.to_css(lightningcss::printer::PrinterOptions {
					minify: true,
					..Default::default()
				})
				.unwrap();
			let css_out = out_dir.join("main.css");
			fs::write(&css_out, css.code).unwrap();
			css_out
		} else {
			"css/main.css".into()
		},
	);

	let assets_out = out_dir.join("assets");
	if embed_assets {
		fs::create_dir_all(&assets_out).unwrap();
	}

	let mut f = fs::File::create(out_dir.join("assets.rs")).unwrap();
	let mut embedded = vec![];

	for (name, path) in assets.assets {
		let out_name = if embed_assets {
			let out_path = assets_out.join(&name);
			fs::copy(assets_dir.join(&path), &out_path).unwrap();
			let contents = fs::read(&out_path).unwrap();
			let hash =
				fast32::base32::CROCKFORD_LOWER.encode(&sha2::Sha256::digest(&contents)[..5]);
			let out_name = match path
				.file_name()
				.expect("should have a file name")
				.to_string_lossy()
				.rsplit_once(".")
			{
				Some((stem, ext)) => format!("{stem}.{hash}.{ext}"),
				None => format!("{name}.{hash}"),
			};

			let mime = mime_guess::from_path(&path).first_or_octet_stream();
			embedded.push((out_name.clone(), mime, name.clone()));

			out_name
		} else {
			path.to_string_lossy().into_owned()
		};

		writeln!(
			f,
			"pub const {name}: &str = {:?};",
			format!("/assets/{out_name}")
		)
		.unwrap();
	}

	println!("cargo::rustc-check-cfg=cfg(pesde_assets_embedded)");
	if !embedded.is_empty() {
		println!("cargo::rustc-cfg=pesde_assets_embedded");
		writeln!(
			f,
			"pub fn get_embedded(name: &str) -> Option<&'static EmbeddedFile> {{",
		)
		.unwrap();
		writeln!(f, "\tmatch name {{",).unwrap();
		for (out_name, mime, name) in embedded {
			writeln!(
				f,
				"\t\t{out_name:?} => Some(&EmbeddedFile {{ content_type: {:?}, contents: include_bytes!({:?}) }}),",
				mime.essence_str(),
				format!("assets/{name}")
			)
			.unwrap();
		}
		writeln!(f, "\t\t_ => None,",).unwrap();
		writeln!(f, "\t}}",).unwrap();
		writeln!(f, "}}",).unwrap();
	}
}

#[derive(Default)]
struct Assets {
	assets: Vec<(String, PathBuf)>,
}

impl Assets {
	pub fn emit(&mut self, name: impl Into<String>, path: impl Into<PathBuf>) {
		self.assets.push((name.into(), path.into()));
	}
}
