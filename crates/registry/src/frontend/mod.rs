use actix_web::{Responder, get, web};
use hypertext::prelude::*;

use crate::util::Env;

mod assets {
	#[cfg(pesde_assets_embedded)]
	pub struct EmbeddedFile {
		pub content_type: &'static str,
		pub contents: &'static [u8],
	}

	include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}

macro_rules! hypertext_include {
    { $( const $name:ident: $src:literal; )* } => {
        $(
            const $name: hypertext::Raw<&'static str> = hypertext::Raw::dangerously_create(include_str!($src));
        )*
    }
}

hypertext_include! {
	const PESDE_LOGO: "./assets/svg/pesde-logo.svg";
}

pub fn config(cfg: &mut web::ServiceConfig) {
	#[cfg(not(pesde_assets_embedded))]
	let assets = actix_files::Files::new(
		"/assets",
		concat!(env!("CARGO_MANIFEST_DIR"), "/src/frontend/assets"),
	);
	#[cfg(pesde_assets_embedded)]
	let assets = embedded_file;

	cfg.service(assets).service(home);
}

#[cfg(pesde_assets_embedded)]
#[get("/assets/{filename:.*}")]
async fn embedded_file(req: actix_web::HttpRequest) -> impl actix_web::Responder {
	let filename = req.match_info().query("filename");
	match assets::get_embedded(filename) {
		Some(file) => actix_web::HttpResponse::Ok()
			.content_type(file.content_type)
			.insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
			.body(file.contents),
		None => actix_web::HttpResponse::NotFound().body(()),
	}
}

#[get("/")]
async fn home() -> impl Responder {
	rsx! {
		<Layout>
			<a href="//daimond113.com">dai</a> sucks
			<button>btw</button>
		</Layout>
	}
}

#[component]
fn root_layout<R: Renderable>(children: &R) -> impl Renderable {
	// script that applies the `dark` class to the root if the user has dark mode enabled.
	let theme_script = hypertext::Raw::dangerously_create(
		"(()=>{\
            const m = window.matchMedia('(prefers-color-scheme: dark)');\
            const u = () => document.documentElement.classList.toggle('dark',\
                localStorage.theme === 'dark' || (!('theme' in localStorage) && m.matches));\
            u();\
            addEventListener('storage',u);\
            addEventListener('pesde:themeChanged',u);\
            m.onchange=u;\
        })()",
	);

	// CSS @font-face declarations
	let font_face = hypertext::Raw::dangerously_create(format!(
		"@font-face {{\
        font-family: 'Nunito Sans Variable';\
        font-style: normal;\
        font-display: swap;\
        font-weight: 200 1000;\
        src: url({}) format('woff2-variations');\
        unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;\
    }}",
		assets::NUNITO_SANS
	));

	rsx! {
		<!DOCTYPE html>
		<html>
			<head>
				<meta charset="UTF-8">
				<meta name="viewport" content="width=device-width, initial-scale=1.0">
				<link rel="stylesheet" href=(assets::CSS)>
				<link rel="preload" href=(assets::NUNITO_SANS) as="font" type="font/woff2" crossorigin />
				<title>hello</title>
				<script>(theme_script)</script>
				<style>(font_face)</style>
			</head>
			<body>
				(children)
			</body>
		</html>
	}
}

#[component]
fn layout<R: Renderable>(children: &R) -> impl Renderable {
	rsx! {
		<RootLayout>
			<header class="navbar">
				<div class="navbar-container">
					<a class="navbar-logo" href="/">(PESDE_LOGO)</a>
					<div class="navbar-right">
						<search>
							<form>
								<input type="text" name="query" />
							</form>
						</search>
						<nav>
							<a href="https://docs.pesde.dev/">docs</a>
							<a href="/policies">policies</a>
						</nav>
						<div>
							<a href="https://github.com/pesde-pkg/pesde">github</a>
						</div>
					</div>
				</div>
			</header>
			<main>
				(children)
			</main>
		</RootLayout>
	}
}
